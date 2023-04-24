use std::mem::size_of;

#[rustfmt::skip]
enum Colors {
    White, Black
}

/**
* Internal piece representation
*     wking=1, wqueen=2, wrook=3, wbishop= 4, wknight= 5, wpawn= 6,
*     bking=7, bqueen=8, brook=9, bbishop=10, bknight=11, bpawn=12
*
* Make sure the piecesyou pass to the library from your engine
* use this format.
*/
#[rustfmt::skip]
enum Pieces {
    Blank = 0, WKing, WQueen, WRook, WBishop, WKnight, WPawn,
               BKing, BQueen, BRook, BBishop, BKnight, BPawn,
}

/**
* nnue data structure
*/
struct DirtyPiece {
    dirty_num: usize,
    pc: [usize; 3],
    from: [usize; 3],
    to: [usize; 3],
}

impl DirtyPiece {
    fn init() -> DirtyPiece {
        DirtyPiece {
            dirty_num: 0,
            pc: [0; 3],
            from: [0; 3],
            to: [0; 3],
        }
    }
}

#[repr(align(64))]
struct Accumulator {
    accumulation: [[i16; 256]; 2],
    computed_accumulation: bool,
}

impl Accumulator {
    fn init() -> Accumulator {
        Accumulator {
            accumulation: [[0; 256]; 2],
            computed_accumulation: false,
        }
    }
}

pub struct NNUEData {
    accumulator: Accumulator,
    dirty_piece: DirtyPiece,
}

impl NNUEData {
    fn init() -> NNUEData {
        NNUEData {
            accumulator: Accumulator::init(),
            dirty_piece: DirtyPiece::init(),
        }
    }
}

/**
* position data structure passed to core subroutines
*  See @nnue_evaluate for a description of parameters
*/
pub struct Position {
    player: usize,
    pieces: [usize; 32],
    squares: [usize; 32],
    nnue: [*mut NNUEData; 3],
}

macro_rules! king {
    ($c: ident) => {
        match $c {
            0 => 1,
            1 => 7,
            _ => $c,
        }
    };
}

macro_rules! is_king {
    ($p: ident) => {
        $p == 1 || $p == 7
    };
}

#[rustfmt::skip]
enum PS {
    WPawn   =  1,
    BPawn   =  1 * 64 + 1,
    WKnight =  2 * 64 + 1,
    BKnight =  3 * 64 + 1,
    WBishop =  4 * 64 + 1,
    BBishop =  5 * 64 + 1,
    WRook   =  6 * 64 + 1,
    BRook   =  7 * 64 + 1,
    WQueen  =  8 * 64 + 1,
    BQueen  =  9 * 64 + 1,
    End     = 10 * 64 + 1,
}

macro_rules! I {
    ($e:expr) => {
        $e as usize
    };
}

#[rustfmt::skip]
const PIECE_TO_INDEX: [[usize; 14]; 2] = [
    [0, 0, I!(PS::WQueen), I!(PS::WRook), I!(PS::WBishop), I!(PS::WKnight), I!(PS::WPawn),
        0, I!(PS::BQueen), I!(PS::BRook), I!(PS::BBishop), I!(PS::BKnight), I!(PS::BPawn), 0],
    [0, 0, I!(PS::BQueen), I!(PS::BRook), I!(PS::BBishop), I!(PS::BKnight), I!(PS::BPawn),
        0, I!(PS::WQueen), I!(PS::WRook), I!(PS::WBishop), I!(PS::WKnight), I!(PS::WPawn), 0],
];

// Version of evaluation file
const NNUE_VERSION: u32 = 0x7AF32F16;

// Constants used in evaluation value calculation
const FV_SCALE: i32 = 16;
const SHIFT: i32 = 6;

const K_HALF_DIMENSIONS: usize = 256;
const FT_IN_DIMS: usize = 64 * I!(PS::End); // 64 * 641
const FT_OUT_DIMS: usize = K_HALF_DIMENSIONS * 2;

const_assert!(K_HALF_DIMENSIONS % 256 == 0);

#[cfg(USE_NEON)]
#[macro_use]
mod neon {
    pub use std::arch::aarch64::*;
    pub const SIMD_WIDTH: usize = 128;
    pub const NUM_REGS: usize = 16;
    pub const TILE_HEIGHT: usize = NUM_REGS * SIMD_WIDTH / 16;
    pub type vec16_t = int16x8_t;
    pub type vec8_t = int8x16_t;
    pub type mask_t = u16;

    pub unsafe fn neon_movemask(v: uint8x16_t) -> mask_t {
        const POWERS: [u8; 16] = [1, 2, 4, 8, 16, 32, 64, 128, 1, 2, 4, 8, 16, 32, 64, 128];
        let k_powers = vld1q_u8(POWERS.as_ptr());

        let mask = vpaddlq_u32(vpaddlq_u16(vpaddlq_u8(vandq_u8(v, k_powers))));
        return vgetq_lane_u8(vreinterpretq_u8_u64(mask), 0) as mask_t
            | (vgetq_lane_u8(vreinterpretq_u8_u64(mask), 8) << 8) as mask_t;
    }
}

#[cfg(USE_AUTO)]
mod auto {
    pub const SIMD_WIDTH: usize = 16; // dummy
    pub type mask_t = u8; // dummy
}

#[cfg(USE_NEON)]
use neon::*;

#[cfg(USE_AUTO)]
use auto::*;

type mask2_t = u64;
type clipped_t = i8;
type weight_t = i8;

struct IndexList {
    size: usize,
    values: [usize; 30],
}

impl IndexList {
    fn init() -> IndexList {
        IndexList {
            size: 0,
            values: [0; 30],
        }
    }
}

#[inline(always)]
fn orient(c: usize, s: usize) -> usize {
    return s ^ if c == 0 { 0x00 } else { 0x3f };
}

#[inline(always)]
fn make_index(c: usize, s: usize, pc: usize, ksq: usize) -> usize {
    return orient(c, s) + PIECE_TO_INDEX[c][pc] + I!(PS::End) * ksq;
}

fn half_kp_append_active_indices(pos: &Position, c: usize, active: &mut IndexList) {
    let mut ksq = pos.squares[c];
    ksq = orient(c, ksq);
    let mut i = 2;
    while pos.pieces[i] != 0 {
        let sq = pos.squares[i];
        let pc = pos.pieces[i];
        active.values[active.size] = make_index(c, sq, pc, ksq);
        active.size += 1;
        i += 1;
    }
}

fn half_kp_append_changed_indices(
    pos: &Position,
    c: usize,
    dp: &DirtyPiece,
    removed: &mut IndexList,
    added: &mut IndexList,
) {
    let mut ksq = pos.squares[c];
    ksq = orient(c, ksq);
    for i in 0..dp.dirty_num {
        let pc = dp.pc[i];
        if is_king!(pc) {
            continue;
        };
        if dp.from[i] != 64 {
            removed.values[removed.size] = make_index(c, dp.from[i], pc, ksq);
            removed.size += 1;
        }
        if dp.to[i] != 64 {
            added.values[added.size] = make_index(c, dp.to[i], pc, ksq);
            added.size += 1;
        }
    }
}

fn append_active_indices(pos: &Position, active: &mut [IndexList; 2]) {
    for c in 0..2 {
        half_kp_append_active_indices(pos, c, &mut active[c])
    }
}

fn append_changed_indices(
    pos: &Position,
    removed: &mut [IndexList; 2],
    added: &mut [IndexList; 2],
    reset: &mut [bool; 2],
) {
    let dp = unsafe { &(*pos.nnue[0]).dirty_piece };

    if unsafe { (*pos.nnue[1]).accumulator.computed_accumulation } {
        for c in 0..2 {
            reset[c] = dp.pc[0] == king!(c);
            if reset[c] {
                half_kp_append_active_indices(pos, c, &mut added[c]);
            } else {
                half_kp_append_changed_indices(pos, c, dp, &mut removed[c], &mut added[c])
            }
        }
    } else {
        let dp2 = unsafe { &(*pos.nnue[1]).dirty_piece };
        for c in 0..2 {
            reset[c] = dp.pc[0] == king!(c) || dp2.pc[0] == king!(c);
            if reset[c] {
                half_kp_append_active_indices(pos, c, &mut added[c]);
            } else {
                half_kp_append_changed_indices(pos, c, dp, &mut removed[c], &mut added[c]);
                half_kp_append_changed_indices(pos, c, dp2, &mut removed[c], &mut added[c]);
            }
        }
    }
}

// InputLayer = InputSlice<256 * 2>
// out: 512 x clipped_t

// Hidden1Layer = ClippedReLu<AffineTransform<InputLayer, 32>>
// 512 x clipped_t -> 32 x int32_t -> 32 x clipped_t

// Hidden2Layer = ClippedReLu<AffineTransform<hidden1, 32>>
// 32 x clipped_t -> 32 x int32_t -> 32 x clipped_t

// OutputLayer = AffineTransform<HiddenLayer2, 1>
// 32 x clipped_t -> 1 x int32_t

pub struct NNUE {
    ft_weights: [i16; K_HALF_DIMENSIONS * FT_IN_DIMS],
    ft_biases: [i16; K_HALF_DIMENSIONS],
    hidden1_weights: [weight_t; 64 * 512],
    hidden1_biases: [i32; 32],
    hidden2_weights: [weight_t; 64 * 32],
    hidden2_biases: [i32; 32],
    output_weights: [weight_t; 1 * 32],
    output_biases: [i32; 1],
}

// Evaluation routines
impl NNUE {
    pub fn evaluate_pos(&self, pos: &mut Position) -> i32 {
        let mut input_mask = [0; FT_OUT_DIMS / (8 * size_of::<mask_t>())];
        let mut hidden1_mask = [0; 8 / size_of::<mask_t>()];
        let mut buf = NetData::init();

        // Input layer
        self.transform(pos, &mut buf.input, &mut input_mask);

        // Hidden 1 Layer
        self.affine_txfm(
            &buf.input,
            &mut buf.hidden1_out,
            FT_OUT_DIMS,
            32,
            &self.hidden1_biases,
            &self.hidden1_weights,
            &input_mask,
            &mut hidden1_mask,
            true,
        );

        // Hidden 2 Layer
        self.affine_txfm(
            &buf.hidden1_out,
            &mut buf.hidden2_out,
            32,
            32,
            &self.hidden2_biases,
            &self.hidden2_weights,
            &hidden1_mask,
            &mut hidden1_mask.clone(), // Dummy
            false,
        );

        // Output layer
        let out_value = self.affine_propagate(buf.hidden2_out);

        return out_value / FV_SCALE;
    }

    /**
     * Evaluation subroutine suitable for chess engines.
     * -------------------------------------------------
     * Piece codes are
     *     wking=1, wqueen=2, wrook=3, wbishop= 4, wknight= 5, wpawn= 6,
     *     bking=7, bqueen=8, brook=9, bbishop=10, bknight=11, bpawn=12,
     * Squares are
     *     A1=0, B1=1 ... H8=63
     * Input format:
     *     piece[0] is white king, square[0] is its location
     *     piece[1] is black king, square[1] is its location
     *     ..
     *     piece[x], square[x] can be in any order
     *     ..
     *     piece[n+1] is set to 0 to represent end of array
     * Returns
     *   Score relative to side to move in approximate centi-pawns
     */
    pub fn evaluate(&self, player: usize, pieces: [usize; 32], squares: [usize; 32]) -> i32 {
        let mut nnue = NNUEData::init();
        let mut pos = Position {
            player,
            pieces,
            squares,
            nnue: [
                &mut nnue as *mut NNUEData,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ],
        };
        return self.evaluate_pos(&mut pos);
    }

    /**
     * Incremental NNUE evaluation function.
     * -------------------------------------------------
     * First three parameters and return type are as in @nnue_evaluate
     *
     * nnue_data
     *    nnue_data[0] is pointer to NNUEdata for ply i.e. current position
     *    nnue_data[1] is pointer to NNUEdata for ply - 1
     *    nnue_data[2] is pointer to NNUEdata for ply - 2
     */
    pub fn evaluate_incremental(
        &self,
        player: usize,
        pieces: [usize; 32],
        squares: [usize; 32],
        nnue: [*mut NNUEData; 3],
    ) -> i32 {
        assert!(
            !nnue[0].is_null()
                && unsafe { std::mem::align_of_val(&(*nnue[0]).accumulator) % 64 == 0 }
        );

        let mut pos = Position {
            player,
            pieces,
            squares,
            nnue,
        };
        return self.evaluate_pos(&mut pos);
    }

    // Convert input features
    fn transform(&self, pos: &mut Position, output: &mut [clipped_t], out_mask: &mut [mask_t]) {
        if unsafe { !self.update_accumulator(pos) } {
            self.refresh_accumulator(pos);
        }

        let accumulation = unsafe { &(*pos.nnue[0]).accumulator.accumulation };
        let perspectives = [pos.player, pos.player ^ 1];
        let mut mask_idx = 0;

        for p in 0..2 {
            let offset = K_HALF_DIMENSIONS * p;

            #[cfg(USE_NEON)]
            unsafe {
                const NUM_CHUNKS: usize = (16 * K_HALF_DIMENSIONS) / SIMD_WIDTH;
                const CHUNK_SIZE: usize = size_of::<vec16_t>() / size_of::<i16>();
                let out: *mut i8 = &mut output[offset];
                for i in 0..(NUM_CHUNKS / 2) {
                    let s0 = vld1q_s16(
                        accumulation[perspectives[p]]
                            .as_ptr()
                            .add(i * 2 * CHUNK_SIZE),
                    );
                    let s1 = vld1q_s16(
                        accumulation[perspectives[p]]
                            .as_ptr()
                            .add((i * 2 + 1) * CHUNK_SIZE),
                    );
                    let out_vec = vcombine_s8(vqmovn_s16(s0), vqmovn_s16(s1));
                    vst1q_s8(out.add(i * size_of::<vec8_t>()), out_vec);
                    out_mask[mask_idx] = neon_movemask(vcgtq_s8(out_vec, vdupq_n_s8(0)));
                    mask_idx += 1;
                }
            }
            #[cfg(USE_AUTO)]
            {
                for i in 0..K_HALF_DIMENSIONS {
                    let sum = accumulation[p][i];
                    output[offset + i] = sum.clamp(0, 127) as i8;
                }
            }
        }
    }

    fn affine_txfm(
        &self,
        input: &[clipped_t],
        output: &mut [clipped_t],
        in_dims: usize,
        out_dims: usize,
        biases: &[i32],
        weights: &[weight_t],
        in_mask: &[mask_t],
        out_mask: &mut [mask_t],
        pack8_and_calc_mask: bool,
    ) {
        #[cfg(USE_NEON)]
        unsafe {
            assert!(out_dims == 32);
            assert!(biases.len() == 32);

            let biases = biases.as_ptr();
            const BIAS_WIDTH: usize = size_of::<int32x4_t>() / size_of::<i32>();
            let mut out_0 = vld1q_s32(biases.add(0 * BIAS_WIDTH));
            let mut out_1 = vld1q_s32(biases.add(1 * BIAS_WIDTH));
            let mut out_2 = vld1q_s32(biases.add(2 * BIAS_WIDTH));
            let mut out_3 = vld1q_s32(biases.add(3 * BIAS_WIDTH));
            let mut out_4 = vld1q_s32(biases.add(4 * BIAS_WIDTH));
            let mut out_5 = vld1q_s32(biases.add(5 * BIAS_WIDTH));
            let mut out_6 = vld1q_s32(biases.add(6 * BIAS_WIDTH));
            let mut out_7 = vld1q_s32(biases.add(7 * BIAS_WIDTH));

            let mut v = 0;

            std::ptr::copy_nonoverlapping(
                // Cast pointer types as u8 so count is interpreted as bytes
                in_mask.as_ptr() as *const u8,
                &mut v as *mut mask2_t as *mut u8,
                size_of::<mask2_t>(),
            );

            let mut idx = 0;
            let mut offset = 0;

            while offset < in_dims {
                if !next_idx(&mut idx, &mut offset, &mut v, in_mask, in_dims) {
                    break;
                }
                let first = weights.as_ptr().add(out_dims * idx);
                let factor = input[idx] as i16;
                let mut prod;
                prod = vmulq_n_s16(vmovl_s8(vld1_s8(first)), factor);
                out_0 = vaddq_s32(out_0, vmovl_s16(vget_low_s16(prod)));
                out_1 = vaddq_s32(out_1, vmovl_high_s16(prod));
                prod = vmulq_n_s16(vmovl_s8(vld1_s8(first.add(8))), factor);
                out_2 = vaddq_s32(out_2, vmovl_s16(vget_low_s16(prod)));
                out_3 = vaddq_s32(out_3, vmovl_high_s16(prod));
                prod = vmulq_n_s16(vmovl_s8(vld1_s8(first.add(16))), factor);
                out_4 = vaddq_s32(out_4, vmovl_s16(vget_low_s16(prod)));
                out_5 = vaddq_s32(out_5, vmovl_high_s16(prod));
                prod = vmulq_n_s16(vmovl_s8(vld1_s8(first.add(24))), factor);
                out_6 = vaddq_s32(out_6, vmovl_s16(vget_low_s16(prod)));
                out_7 = vaddq_s32(out_7, vmovl_high_s16(prod));
            }

            let out_16_0 = vcombine_s16(vqshrn_n_s32(out_0, SHIFT), vqshrn_n_s32(out_1, SHIFT));
            let out_16_1 = vcombine_s16(vqshrn_n_s32(out_2, SHIFT), vqshrn_n_s32(out_3, SHIFT));
            let out_16_2 = vcombine_s16(vqshrn_n_s32(out_4, SHIFT), vqshrn_n_s32(out_5, SHIFT));
            let out_16_3 = vcombine_s16(vqshrn_n_s32(out_6, SHIFT), vqshrn_n_s32(out_7, SHIFT));

            if pack8_and_calc_mask {
                let k_zero = vld1q_dup_s8(0 as *const i8);
                let out_vec = output.as_mut_ptr();
                let out0_vec = vcombine_s8(vqmovn_s16(out_16_0), vqmovn_s16(out_16_1));
                vst1q_s8(out_vec, out0_vec);
                out_mask[0] = neon_movemask(vcgtq_s8(out0_vec, k_zero));
                let out1_vec = vcombine_s8(vqmovn_s16(out_16_2), vqmovn_s16(out_16_3));
                vst1q_s8(out_vec.add(size_of::<int8x16_t>()), out1_vec);
                out_mask[1] = neon_movemask(vcgtq_s8(out1_vec, k_zero));
            } else {
                // The next step takes int8x8_t as input, so store as int8x8_t
                let k_zero = vld1_dup_s8(0 as *const i8);
                let out_vec = output.as_mut_ptr();
                vst1_s8(out_vec, vmax_s8(vqmovn_s16(out_16_0), k_zero));
                vst1_s8(out_vec.add(8), vmax_s8(vqmovn_s16(out_16_1), k_zero));
                vst1_s8(out_vec.add(16), vmax_s8(vqmovn_s16(out_16_2), k_zero));
                vst1_s8(out_vec.add(24), vmax_s8(vqmovn_s16(out_16_3), k_zero));
            }
        }
        #[cfg(USE_AUTO)]
        {
            let mut tmp = vec![0i32; out_dims];

            for i in 0..out_dims {
                tmp[i] = biases[i]
            }

            for idx in 0..in_dims {
                if input[idx] != 0 {
                    for i in 0..out_dims {
                        tmp[i] += input[idx] as i32 * weights[out_dims * idx + i] as i32
                    }
                }
            }

            for i in 0..out_dims {
                output[i] = (tmp[i] >> SHIFT).max(0).min(127) as clipped_t;
            }
        }
    }

    fn affine_propagate(&self, input: [clipped_t; 32]) -> i32 {
        #[cfg(USE_NEON)]
        unsafe {
            let iv = vld1_s8_x4(input.as_ptr());
            let mut sum = vld1q_dup_s32(self.output_biases.as_ptr());
            let row = vld1_s8_x4(self.output_weights.as_ptr());
            let mut p0 = vmull_s8(iv.0, row.0);
            let mut p1 = vmull_s8(iv.1, row.1);
            p0 = vmlal_s8(p0, iv.2, row.2);
            sum = vpadalq_s16(sum, p0);
            p1 = vmlal_s8(p1, iv.3, row.3);
            sum = vpadalq_s16(sum, p1);
            return vaddvq_s32(sum);
        }
        #[cfg(USE_AUTO)]
        {
            let mut sum = biases[0];
            for j in 0..32 {
                sum += weights[j] as i32 * input[j] as i32;
            }
            return sum;
        }
    }

    // Calculate cumulative value without using difference calculation
    fn refresh_accumulator(&self, pos: &mut Position) {
        let mut active_indices = [IndexList::init(), IndexList::init()];
        append_active_indices(pos, &mut active_indices);

        let accumulator = unsafe { &mut (*pos.nnue[0]).accumulator };

        #[cfg(USE_NEON)]
        unsafe {
            const VSIZE: usize = size_of::<vec16_t>() / size_of::<u16>();
            for c in 0..2 {
                for i in 0..(K_HALF_DIMENSIONS / TILE_HEIGHT) {
                    let ft_biases_tile = self.ft_biases.as_ptr().add(i * TILE_HEIGHT);
                    let acc_tile = accumulator.accumulation[c]
                        .as_mut_ptr()
                        .add(i * TILE_HEIGHT);
                    let mut acc = Vec::with_capacity(NUM_REGS);

                    for j in 0..NUM_REGS {
                        acc.push(vld1q_s16(ft_biases_tile.add(VSIZE * j)))
                    }

                    for k in 0..active_indices[c].size {
                        let index = active_indices[c].values[k];
                        let offset = K_HALF_DIMENSIONS * index + i * TILE_HEIGHT;
                        let column = self.ft_weights.as_ptr().add(offset);

                        for j in 0..NUM_REGS {
                            acc[j] = vaddq_s16(acc[j], vld1q_s16(column.add(VSIZE * j)));
                        }
                    }

                    for j in 0..NUM_REGS {
                        vst1q_s16(acc_tile.add(VSIZE * j), acc[j])
                    }
                }
            }
        }
        #[cfg(USE_AUTO)]
        for c in 0..2 {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.ft_biases.as_ptr(),
                    accumulator.accumulation[c].as_mut_ptr(),
                    K_HALF_DIMENSIONS,
                );
            }
            for k in 0..active_indices[c].size {
                let index = active_indices[c].values[k];
                let offset = K_HALF_DIMENSIONS * index;

                for j in 0..K_HALF_DIMENSIONS {
                    accumulator.accumulation[c][j] += self.ft_weights[offset + j];
                }
            }
        }

        accumulator.computed_accumulation = true;
    }

    // Calculate cumulative value using difference calculation if possible
    unsafe fn update_accumulator(&self, pos: &mut Position) -> bool {
        let pos_ptr: *mut Position = pos;
        let accumulator: *mut Accumulator = &mut (*(*pos_ptr).nnue[0]).accumulator;
        if (*accumulator).computed_accumulation {
            return true;
        }

        let mut prev_acc = std::ptr::null::<Accumulator>();

        if (pos.nnue[1].is_null()
            || (*{
                prev_acc = &(*pos.nnue[1]).accumulator;
                prev_acc
            })
            .computed_accumulation)
            || (pos.nnue[2].is_null()
                || (*{
                    prev_acc = &(*pos.nnue[2]).accumulator;
                    prev_acc
                })
                .computed_accumulation)
        {
            return false;
        }

        assert!(!prev_acc.is_null());

        let mut removed_indices = [IndexList::init(), IndexList::init()];
        let mut added_indices = [IndexList::init(), IndexList::init()];
        let mut reset = [false; 2];
        append_changed_indices(pos, &mut removed_indices, &mut added_indices, &mut reset);

        #[cfg(USE_NEON)]
        {
            const VSIZE: usize = size_of::<vec16_t>() / size_of::<u16>();
            for i in 0..K_HALF_DIMENSIONS / TILE_HEIGHT {
                for c in 0..2 {
                    let acc_tile = (*accumulator).accumulation[c]
                        .as_mut_ptr()
                        .add(i * TILE_HEIGHT);
                    let mut acc = Vec::with_capacity(NUM_REGS);

                    if reset[c] {
                        let ft_b_tile = self.ft_biases.as_ptr().add(i * TILE_HEIGHT);
                        for j in 0..NUM_REGS {
                            acc.push(vld1q_s16(ft_b_tile.add(j * VSIZE)))
                        }
                    } else {
                        let prev_acc_tile =
                            (*prev_acc).accumulation[c].as_ptr().add(i * TILE_HEIGHT);
                        for j in 0..NUM_REGS {
                            acc.push(vld1q_s16(prev_acc_tile.add(j * VSIZE)))
                        }

                        // Difference calculation for the deactivated features
                        for k in 0..removed_indices[c].size {
                            let index = removed_indices[c].values[k];
                            let offset = K_HALF_DIMENSIONS * index + i * TILE_HEIGHT;

                            let column = self.ft_weights.as_ptr().add(offset);
                            for j in 0..NUM_REGS {
                                acc[j] = vsubq_s16(acc[j], vld1q_s16(column.add(j * VSIZE)));
                            }
                        }

                        for j in 0..NUM_REGS {
                            vst1q_s16(acc_tile.add(j * VSIZE), acc[j])
                        }
                    }
                }
            }
        }

        #[cfg(USE_AUTO)]
        {
            for c in 0..2 {
                if reset[c] {
                    std::ptr::copy_nonoverlapping(
                        nn.ft_biases.as_ptr(),
                        (*accumulator).accumulation[c].as_mut_ptr(),
                        K_HALF_DIMENSIONS,
                    )
                } else {
                    std::ptr::copy_nonoverlapping(
                        (*prev_acc).accumulation[c].as_ptr(),
                        (*accumulator).accumulation[c].as_mut_ptr(),
                        K_HALF_DIMENSIONS,
                    );
                    // Difference calculation for the deactivated features
                    for k in 0..removed_indices[c].size {
                        let index = removed_indices[c].values[k];
                        let offset = K_HALF_DIMENSIONS * index;

                        for j in 0..K_HALF_DIMENSIONS {
                            (*accumulator).accumulation[c][j] -= nn.ft_weights[offset + j]
                        }
                    }
                }

                // Difference calculation for the activated features
                for k in 0..added_indices[c].size {
                    let index = added_indices[c].values[k];
                    let offset = K_HALF_DIMENSIONS * index;

                    for j in 0..K_HALF_DIMENSIONS {
                        (*accumulator).accumulation[c][j] += nn.ft_weights[offset + j];
                    }
                }
            }
        }

        (*accumulator).computed_accumulation = true;
        return true;
    }
}

#[cfg(USE_NEON)]
fn next_idx(
    idx: &mut usize,
    offset: &mut usize,
    v: &mut mask2_t,
    mask: &[mask_t],
    in_dims: usize,
) -> bool {
    while *v == 0 {
        *offset += 8 * size_of::<mask2_t>();
        if *offset >= in_dims {
            return false;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                (mask.as_ptr() as *const u8).add(*offset / 8),
                v as *mut mask2_t as *mut u8,
                size_of::<mask2_t>(),
            )
        };
    }
    *idx = *offset + v.trailing_zeros() as usize;
    *v &= *v - 1;
    return true;
}

#[repr(align(64))]
struct NetData {
    input: [clipped_t; FT_OUT_DIMS],
    hidden1_out: [clipped_t; 32],
    hidden2_out: [clipped_t; 32],
}

impl NetData {
    fn init() -> NetData {
        NetData {
            input: [0; FT_OUT_DIMS],
            hidden1_out: [0; 32],
            hidden2_out: [0; 32],
        }
    }
}

// Initialization routines
impl NNUE {
    pub fn init(eval_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Loading NNUE : {eval_file}");
        let nn = NNUE {
            ft_weights: [0; K_HALF_DIMENSIONS * FT_IN_DIMS],
            ft_biases: [0; K_HALF_DIMENSIONS],
            hidden1_weights: [0; 64 * 512],
            hidden1_biases: [0; 32],
            hidden2_weights: [0; 64 * 32],
            hidden2_biases: [0; 32],
            output_weights: [0; 1 * 32],
            output_biases: [0; 1],
        };

        nn.load_eval_file(eval_file)?;
        return Ok(());
    }

    fn load_eval_file(&self, eval_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}

fn read_output_weights(w: &mut [weight_t], d: &[u8]) {
    for i in 0..32 {
        w[i] = d[i] as weight_t;
    }
}

#[inline(always)]
fn wt_idx(r: usize, c: usize, dims: usize) -> usize {
    return c * 32 + r;
}

fn read_hidden_weights<'a>(w: &'a mut [weight_t], dims: usize, d: &'a [u8]) -> &'a [u8] {
    let mut d = d;
    for r in 0..32 {
        for c in 0..dims {
            w[wt_idx(r, c, dims)] = d[0] as weight_t;
            d = &d[1..];
        }
    }
    return d;
}

const TRANSFORMER_START: usize = 3 * 4 + 177;
const NETWORK_START: usize = TRANSFORMER_START + 4 + 2 * 256 + 2 * 256 * 64 * 641;
