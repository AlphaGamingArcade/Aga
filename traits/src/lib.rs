// The Licensed Work is (c) 2022 Sygma
// SPDX-License-Identifier: LGPL-3.0-only

#![cfg_attr(not(feature = "std"), no_std)]

use sp_core::{H160, U256};
use scale_info::TypeInfo;
use scale_info::prelude::vec::Vec;
use codec::{Decode, Encode};

pub type DomainID = u8;
pub type DepositNonce = u64;
pub type ResourceId = [u8; 32];
pub type ChainID = U256;
pub type VerifyingContractAddress = H160;
pub type DestChainRecipient = Vec<u8>;
pub type DestinationLocation = (Vec<u8>, DomainID);


#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, TypeInfo)]
pub enum TransferType {
	FungibleTransfer,
	NonFungibleTransfer,
	GenericTransfer,
}