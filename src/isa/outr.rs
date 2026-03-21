// Reference rust implementation of AluVM (arithmetic logic unit virtual machine).
// To find more on AluVM please check <https://aluvm.org>
//
// SPDX-License-Identifier: Apache-2.0
//
// Copyright (C) 2021-2022 LNP/BP Standards Association. All rights reserved.
// Copyright (C) 2023-2024 UBIDECO Labs,
//     Institute for Distributed and Cognitive Computing, Switzerland.
//     All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! OUTR extension: append-only output of A64 / S16 register values (Outstack ISA, opcode `0x90`).

use alloc::vec::Vec;

use core::cell::RefCell;
use core::ops::RangeInclusive;

use super::bytecode::{Bytecode, BytecodeError};
use super::opcodes::{INSTR_OUTR, INSTR_OUTSTACK_FROM, INSTR_OUTSTACK_TO};
use crate::library::{CodeEofError, Read, Write};

/// One value produced by [`RgbExt::Outr`](RgbExt::Outr).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum OutrValue {
    /// Integer from `a64[reg]` (0..32).
    Int(i64),
    /// Bytes from `s16[reg - 32]` (32..48).
    Bytes(Vec<u8>),
}

/// Host-provided output buffer and limits for OUTR execution.
///
/// Uses [`RefCell`] so [`InstructionSet::exec`](super::InstructionSet::exec) can append while the
/// VM passes the context by shared reference.
pub struct OutrContext<'a> {
    /// Append-only output stack.
    pub output: &'a RefCell<Vec<OutrValue>>,
    /// Maximum number of items; exceeding it fails the program.
    pub max_items: usize,
}

/// Outstack extension instructions decoded as [`Instr::ExtensionCodes`](super::Instr).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Display)]
#[display(inner)]
pub enum RgbExt {
    /// `OUTR(reg)`: reg 0..32 outputs `a64[reg]`; reg 32..48 outputs `s16[reg - 32]`.
    #[display("outr    {0}")]
    Outr(u8),
}

impl RgbExt {
    fn decode_outr_after_opcode<R: Read>(reader: &mut R) -> Result<Self, CodeEofError> {
        Ok(RgbExt::Outr(reader.read_u8()?))
    }
}

impl Bytecode for RgbExt {
    #[inline]
    fn instr_range() -> RangeInclusive<u8> { INSTR_OUTSTACK_FROM..=INSTR_OUTSTACK_TO }

    fn instr_byte(&self) -> u8 { INSTR_OUTR }

    fn encode_args<W>(&self, writer: &mut W) -> Result<(), BytecodeError>
    where W: Write {
        let RgbExt::Outr(reg) = self;
        writer.write_u8(*reg)?;
        Ok(())
    }

    fn decode<R>(reader: &mut R) -> Result<Self, CodeEofError>
    where R: Read {
        let opcode = reader.read_u8()?;
        match opcode {
            INSTR_OUTR => Self::decode_outr_after_opcode(reader),
            x if (INSTR_OUTSTACK_FROM..=INSTR_OUTSTACK_TO).contains(&x) => Err(CodeEofError),
            x => unreachable!("instruction {:#010b} classified as RgbExt extension", x),
        }
    }
}
