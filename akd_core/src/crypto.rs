// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is dual-licensed under either the MIT license found in the
// LICENSE-MIT file in the root directory of this source tree or the Apache
// License, Version 2.0 found in the LICENSE-APACHE file in the root directory
// of this source tree. You may select, at your option, one of the above-listed licenses.

//! Functions for performing the core cryptographic operations for AKD

use crate::hash::{hash, Digest, DIGEST_BYTES};
use crate::utils::i2osp_array;
use crate::{
    AkdLabel, AkdValue, AzksValue, AzksValueWithEpoch, NodeLabel, VersionFreshness, EMPTY_LABEL,
    EMPTY_VALUE,
};

#[cfg(feature = "nostd")]
use alloc::vec::Vec;

/// The value stored in the root node upon initialization, with no children
pub fn empty_root_value() -> AzksValue {
    // FIXME(#344) Change this to:
    // [0u8; 32]
    AzksValue(hash(&crate::EMPTY_VALUE))
}

/// AZKS value corresponding to an empty node
pub fn empty_node_hash() -> AzksValue {
    // FIXME(#344) Change this to:
    // [0u8; 32]
    AzksValue(hash(
        &[hash(&EMPTY_VALUE).to_vec(), EMPTY_LABEL.value()].concat(),
    ))
}

/// Used by the client to supply a commitment nonce and value to reconstruct the commitment, via:
/// commitment = H(i2osp_array(value), i2osp_array(nonce))
pub(crate) fn generate_commitment_from_nonce_client(
    value: &crate::AkdValue,
    nonce: &[u8],
) -> AzksValue {
    AzksValue(hash(&[i2osp_array(value), i2osp_array(nonce)].concat()))
}

/// Hash a leaf epoch and nonce with a given [AkdValue]
pub(crate) fn hash_leaf_with_value(
    value: &crate::AkdValue,
    epoch: u64,
    nonce: &[u8],
) -> AzksValueWithEpoch {
    let commitment = generate_commitment_from_nonce_client(value, nonce);
    hash_leaf_with_commitment(commitment, epoch)
}

/// Hash a commit and epoch together to get the leaf's hash value
pub fn hash_leaf_with_commitment(commitment: AzksValue, epoch: u64) -> AzksValueWithEpoch {
    let mut data = [0; DIGEST_BYTES + 8];
    data[..DIGEST_BYTES].copy_from_slice(&commitment.0);
    data[DIGEST_BYTES..].copy_from_slice(&epoch.to_be_bytes());
    AzksValueWithEpoch(hash(&data))
}

/// Used by the server to produce a commitment nonce for an AkdLabel, version, and AkdValue.
/// Computes nonce = H(commitment key || label)
pub fn get_commitment_nonce(
    commitment_key: &[u8],
    label: &NodeLabel,
    version: u64,
    value: &AkdValue,
) -> Digest {
    // FIXME(#344) Change this to:
    // hash(
    //    &[
    //        commitment_key,
    //        &label.to_bytes(),
    //    ]
    //    .concat(),
    //)
    hash(
        &[
            commitment_key,
            &label.to_bytes(),
            &version.to_be_bytes(),
            &i2osp_array(value),
        ]
        .concat(),
    )
}

/// To convert a regular label (arbitrary string of bytes) into a [NodeLabel], we compute the
/// output as: H(label || freshness || version)
///
/// Specifically, we concatenate the following together:
/// - I2OSP(len(label) as u64, label)
/// - A single byte encoded as 0u8 if "stale", 1u8 if "fresh"
/// - A u64 representing the version
/// These are all interpreted as a single byte array and hashed together, with the output
/// of the hash returned.
pub(crate) fn get_hash_from_label_input(
    label: &AkdLabel,
    freshness: VersionFreshness,
    version: u64,
) -> Vec<u8> {
    let freshness_bytes = [freshness as u8];
    let hashed_label = hash(
        &[
            &crate::utils::i2osp_array(label)[..],
            &freshness_bytes,
            &version.to_be_bytes(),
        ]
        .concat(),
    );
    hashed_label.to_vec()
}

/// Computes the parent hash from the children hashes and labels
pub fn compute_parent_hash_from_children(
    left_val: &AzksValue,
    left_label: &[u8],
    right_val: &AzksValue,
    right_label: &[u8],
) -> AzksValue {
    // FIXME(#344) Change this to:
    // hash(
    //    &[
    //        left_val,
    //        left_label,
    //        right_val,
    //        right_label,
    //    ].concat()
    // )
    AzksValue(hash(
        &[
            hash(&[left_val.0.to_vec(), left_label.to_vec()].concat()),
            hash(&[right_val.0.to_vec(), right_label.to_vec()].concat()),
        ]
        .concat(),
    ))
}

/// Given the top-level hash, compute the "actual" root hash that is published
/// by the directory maintainer
pub fn compute_root_hash_from_val(root_val: &AzksValue) -> Digest {
    // FIXME(#344) Change this to:
    // root_val
    hash(&[&root_val.0[..], &NodeLabel::root().value()].concat())
}

/// Used by the server to produce a commitment for an AkdLabel, version, and AkdValue
///
/// nonce = H(commitment_key, label, version, i2osp_array(value))
/// commmitment = H(i2osp_array(value), i2osp_array(nonce))
///
/// The nonce value is used to create a hiding and binding commitment using a
/// cryptographic hash function. Note that it is derived from the label, version, and
/// value (even though the binding to value is somewhat optional).
///
/// Note that this commitment needs to be a hash function (random oracle) output
pub fn compute_fresh_azks_value(
    commitment_key: &[u8],
    label: &NodeLabel,
    version: u64,
    value: &AkdValue,
) -> AzksValue {
    let nonce = get_commitment_nonce(commitment_key, label, version, value);
    AzksValue(hash(&[i2osp_array(value), i2osp_array(&nonce)].concat()))
}

/// Similar to commit_fresh_value, but used for stale values.
pub fn stale_azks_value() -> AzksValue {
    // FIXME(#344) Change this to:
    // crate::hash::EMPTY_DIGEST
    AzksValue(hash(&EMPTY_VALUE))
}
