/*
Copyright 2023 The Kubernetes Authors.

SPDX-License-Identifier: (GPL-2.0-only OR BSD-2-Clause)
*/

use core::mem;

use aya_bpf::{bindings::TC_ACT_PIPE, helpers::bpf_csum_diff, programs::TcContext};
use aya_log_ebpf::info;
use network_types::{eth::EthHdr, icmp::IcmpHdr, ip::Ipv4Hdr};

use crate::{
    utils::{csum_fold_helper, ptr_at},
    BLIXT_CONNTRACK,
};

const ICMP_PROTO_TYPE_UNREACH: u8 = 3;

pub fn handle_icmp_egress(ctx: TcContext) -> Result<i32, i64> {
    let ip_hdr: *mut Ipv4Hdr = unsafe { ptr_at(&ctx, EthHdr::LEN)? };

    let icmp_header_offset = EthHdr::LEN + Ipv4Hdr::LEN;

    let icmp_hdr: *mut IcmpHdr = unsafe { ptr_at(&ctx, icmp_header_offset)? };

    // We only care about redirecting port unreachable messages currently so a
    // UDP client can tell when the server is shutdown
    if unsafe { (*icmp_hdr).type_ } != ICMP_PROTO_TYPE_UNREACH {
        return Ok(TC_ACT_PIPE);
    }

    let dest_addr = unsafe { (*ip_hdr).dst_addr };

    let new_src = unsafe { BLIXT_CONNTRACK.get(&dest_addr) }.ok_or(TC_ACT_PIPE)?;

    info!(
        &ctx,
        "Received a ICMP Unreachable packet destined for svc ip: {:i} ",
        u32::from_be(unsafe { (*ip_hdr).dst_addr })
    );

    // redirect icmp unreachable message back to client
    unsafe {
        (*ip_hdr).src_addr = new_src.0;
        (*ip_hdr).check = 0;
    }

    let full_cksum = unsafe {
        bpf_csum_diff(
            mem::MaybeUninit::zeroed().assume_init(),
            0,
            ip_hdr as *mut u32,
            Ipv4Hdr::LEN as u32,
            0,
        )
    } as u64;
    unsafe { (*ip_hdr).check = csum_fold_helper(full_cksum) };

    // Get inner ipheader since we need to update that as well
    let icmp_inner_ip_hdr: *mut Ipv4Hdr =
        unsafe { ptr_at(&ctx, icmp_header_offset + IcmpHdr::LEN) }?;

    unsafe {
        (*icmp_inner_ip_hdr).dst_addr = new_src.0;
        (*icmp_inner_ip_hdr).check = 0;
    }

    let full_cksum = unsafe {
        bpf_csum_diff(
            mem::MaybeUninit::zeroed().assume_init(),
            0,
            icmp_inner_ip_hdr as *mut u32,
            Ipv4Hdr::LEN as u32,
            0,
        )
    } as u64;
    unsafe { (*icmp_inner_ip_hdr).check = csum_fold_helper(full_cksum) };

    unsafe { BLIXT_CONNTRACK.remove(&dest_addr)? };

    return Ok(TC_ACT_PIPE);
}
