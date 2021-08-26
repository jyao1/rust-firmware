// Copyright (c) 2021 Intel Corporation
//
// SPDX-License-Identifier: BSD-2-Clause-Patent

use crate::Result;

pub trait RxToken {
    /// Consumes the token to receive a single packet.
    ///
    /// This method receives a packet and then calls the given closure `f` with the raw
    /// packet bytes as argument.
    fn consume<R, F>(self, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>;
}

/// A token to transmit a single packet.
pub trait TxToken {
    /// Consumes the token to send a single packet.
    fn consume<R, F>(self, len: usize, f: F) -> Result<R>
    where
        F: FnOnce(&mut [u8]) -> Result<R>;
}

pub trait Device<'a> {
    type RxToken: RxToken + 'a;
    type TxToken: TxToken + 'a;

    /// Construct a token pair consisting of one receive token and one transmit token.
    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)>;

    /// Construct a transmit token.
    fn transmit(&'a mut self) -> Option<Self::TxToken>;
}
