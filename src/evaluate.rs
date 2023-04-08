use super::*;

use position::Position;
use types::PieceType;

const WHITE: usize = 0;
const BLACK: usize = 1;
const COLORS: [usize; 2] = [WHITE, BLACK];

const MG: usize = 0;
const EG: usize = 1;
const PHASES: [usize; 2] = [MG, EG];

const PAWN: usize = 0;
const ROOK: usize = 1;
const KNIGHT: usize = 2;
const BISHOP: usize = 3;
const QUEEN: usize = 4;
const KING: usize = 5;
const PIECES: [usize; 6] = [PAWN, ROOK, KNIGHT, BISHOP, QUEEN, KING];

/* The phase score allows interpolation between the middlegame and endgame psqts.
   It's scored from 0, a pure endgame, to 24, a pure middlegame.
*/
const PHASE_CONTRIBUTIONS: [i32; 6] = [0, 2, 1, 1, 4, 0];
const MAX_MG_PHASE: i32 = 24;

#[derive(Clone, Copy)]
pub struct Score {
    mg: i32,
    eg: i32,
    phase: i32,
}

impl Score {
    pub fn new_equal() -> Self {
        Self {
            mg: 0,
            eg: 0,
            phase: 0,
        }
    }
}

impl Position {
    pub fn init_score(&mut self) {
        let (mut white, mut black) = (self.us, self.them);
        if !self.wtm {
            std::mem::swap(&mut white, &mut black)
        };
        let bbsets = [&white.as_array(), &black.as_array()];

        for stm in COLORS {
            let sign = if stm == WHITE { 1 } else { -1 }; // Black eval is negative
            for pt in PIECES {
                let bb = bbsets[stm][pt + 1];
                for sq in *bb {
                    self.score.mg += SCORES[stm][pt][MG][sq.to_sq()] * sign;
                    self.score.eg += SCORES[stm][pt][EG][sq.to_sq()] * sign;
                    self.score.phase += PHASE_CONTRIBUTIONS[pt];
                }
            }
        }
    }

    pub fn score_move_update(&mut self, pt: PieceType, from: BB, to: BB) {
        let sign = if self.wtm { 1 } else { -1 };
        let stm = self.stm as usize;
        let pt = pt as usize - 1;
        let from = from.to_sq();
        let to = to.to_sq();

        // Remove score from previous sq
        self.score.mg -= SCORES[stm][pt][MG][from] * sign;
        self.score.eg -= SCORES[stm][pt][EG][from] * sign;

        // Add score for new sq
        self.score.mg += SCORES[stm][pt][MG][to] * sign;
        self.score.eg += SCORES[stm][pt][EG][to] * sign;
    }

    pub fn score_capture_update(&mut self, pt: PieceType, sq: BB) {
        let sign = if self.wtm { 1 } else { -1 };
        let not_stm = self.wtm as usize;
        let pt = pt as usize - 1;
        let sq = sq.to_sq();

        // Add score for side currently moving
        self.score.mg += SCORES[not_stm][pt][MG][sq] * sign;
        self.score.eg += SCORES[not_stm][pt][EG][sq] * sign;

        // Update phase
        self.score.phase -= PHASE_CONTRIBUTIONS[pt]
    }

    pub fn score_promotion_update(&mut self, pt: PieceType, sq: BB) {
        let sign = if self.wtm { 1 } else { -1 };
        let stm = self.stm as usize;
        let pt = pt as usize - 1;
        let sq = sq.to_sq();

        // Unset the pawn
        self.score.mg -= SCORES[stm][PAWN][MG][sq] * sign;
        self.score.eg -= SCORES[stm][PAWN][EG][sq] * sign;

        // Set the promotion piece
        self.score.mg += SCORES[stm][pt][MG][sq] * sign;
        self.score.eg += SCORES[stm][pt][EG][sq] * sign;

        // Update phase
        self.score.phase += PHASE_CONTRIBUTIONS[pt];
    }

    pub fn evaluate(&self) -> i16 {
        // Use a maximum phase of 24, in case of early promotion
        let phase = std::cmp::min(MAX_MG_PHASE, self.score.phase);

        return ((self.score.mg * phase + self.score.eg * (MAX_MG_PHASE - phase)) / MAX_MG_PHASE)
            as i16;
    }
}

lazy_static! {
    /* Initialize the score lookup table with the index scheme:
            color->piecetype->gamephase->square->score
       The score is a combined material and positional evaluation
    */
    static ref SCORES: [[[[i32; 64]; 2]; 6]; 2] = {
        let mut scores = [[[[0; 64]; 2]; 6]; 2];

        for pt in PIECES {
            for stm in COLORS {
                for phase in PHASES {
                    for sq in 0..64 {
                        let material = tables::MATERIAL[pt][phase];
                        let position = if stm == WHITE {
                            tables::PSQT[pt][phase][sq^56]
                        } else {
                            tables::PSQT[pt][phase][sq]
                        };
                        scores[stm][pt][phase][sq] += material + position;
                    }
                }
            }
        }
        scores
    };
}

#[rustfmt::skip]
mod tables {
    // Taken from the chess wiki "PeSTO" evaluation function.
    pub const PSQT: [[[i32; 64]; 2]; 6] = [
        [ // Pawn
            [ // MG
                   0,   0,   0,   0,   0,   0,   0,   0,
                  98, 134,  61,  95,  68, 126,  34, -11,
                  -6,   7,  26,  31,  65,  56,  25, -20,
                 -14,  13,   6,  21,  23,  12,  17, -23,
                 -27,  -2,  -5,  12,  17,   6,  10, -25,
                 -26,  -4,  -4, -10,   3,   3,  33, -12,
                 -35,  -1, -20, -23, -15,  24,  38, -22,
                   0,   0,   0,   0,   0,   0,   0,   0,
            ],
    
            [ // EG
                   0,   0,   0,   0,   0,   0,   0,   0,
                 178, 173, 158, 134, 147, 132, 165, 187,
                  94, 100,  85,  67,  56,  53,  82,  84,
                  32,  24,  13,   5,  -2,   4,  17,  17,
                  13,   9,  -3,  -7,  -7,  -8,   3,  -1,
                   4,   7,  -6,   1,   0,  -5,  -1,  -8,
                  13,   8,   8,  10,  13,   0,   2,  -7,
                   0,   0,   0,   0,   0,   0,   0,   0,
            ]
        ],
    
        [ // Rook
            [ // MG
                  32,  42,  32,  51,  63,   9,  31,  43,
                  27,  32,  58,  62,  80,  67,  26,  44,
                  -5,  19,  26,  36,  17,  45,  61,  16,
                 -24, -11,   7,  26,  24,  35,  -8, -20,
                 -36, -26, -12,  -1,   9,  -7,   6, -23,
                 -45, -25, -16, -17,   3,   0,  -5, -33,
                 -44, -16, -20,  -9,  -1,  11,  -6, -71,
                 -19, -13,   1,  17,  16,   7, -37, -26,
            ],
    
            [ // EG
                  13,  10,  18,  15,  12,  12,   8,   5,
                  11,  13,  13,  11,  -3,   3,   8,   3,
                   7,   7,   7,   5,   4,  -3,  -5,  -3,
                   4,   3,  13,   1,   2,   1,  -1,   2,
                   3,   5,   8,   4,  -5,  -6,  -8, -11,
                  -4,   0,  -5,  -1,  -7, -12,  -8, -16,
                  -6,  -6,   0,   2,  -9,  -9, -11,  -3,
                  -9,   2,   3,  -1,  -5, -13,   4, -20,
            ]
        ],
    
        [ // Knight
            [ // MG
                -167, -89, -34, -49,  61, -97, -15,-107,
                 -73, -41,  72,  36,  23,  62,   7, -17,
                 -47,  60,  37,  65,  84, 129,  73,  44,
                  -9,  17,  19,  53,  37,  69,  18,  22,
                 -13,   4,  16,  13,  28,  19,  21,  -8,
                 -23,  -9,  12,  10,  19,  17,  25, -16,
                 -29, -53, -12,  -3,  -1,  18, -14, -19,
                -105, -21, -58, -33, -17, -28, -19, -23,
            ],
    
            [ // EG
                 -58, -38, -13, -28, -31, -27, -63, -99,
                 -25,  -8, -25,  -2,  -9, -25, -24, -52,
                 -24, -20,  10,   9,  -1,  -9, -19, -41,
                 -17,   3,  22,  22,  22,  11,   8, -18,
                 -18,  -6,  16,  25,  16,  17,   4, -18,
                 -23,  -3,  -1,  15,  10,  -3, -20, -22,
                 -42, -20, -10,  -5,  -2, -20, -23, -44,
                 -29, -51, -23, -15, -22, -18, -50, -64,
            ]
        ],
    
        [ // Bishop
            [ // MG
                - 29,   4, -82, -37, -25, -42,   7,  -8,
                - 26,  16, -18, -13,  30,  59,  18, -47,
                - 16,  37,  43,  40,  35,  50,  37,  -2,
                  -4,   5,  19,  50,  37,  37,   7,  -2,
                  -6,  13,  13,  26,  34,  12,  10,   4,
                   0,  15,  15,  15,  14,  27,  18,  10,
                   4,  15,  16,   0,   7,  21,  33,   1,
                - 33,  -3, -14, -21, -13, -12, -39, -21,
            ],
    
            [ // EG
                 -14, -21, -11,  -8,  -7,  -9, -17, -24,
                  -8,  -4,   7, -12,  -3, -13,  -4, -14,
                   2,  -8,   0,  -1,  -2,   6,   0,   4,
                  -3,   9,  12,   9,  14,  10,   3,   2,
                  -6,   3,  13,  19,   7,  10,  -3,  -9,
                 -12,  -3,   8,  10,  13,   3,  -7, -15,
                 -14, -18,  -7,  -1,   4,  -9, -15, -27,
                 -23,  -9, -23,  -5,  -9, -16,  -5, -17,
            ]
        ],
    
        [ // Queen
            [ // MG
                 -28,   0,  29,  12,  59,  44,  43,  45,
                 -24, -39,  -5,   1, -16,  57,  28,  54,
                 -13, -17,   7,   8,  29,  56,  47,  57,
                 -27, -27, -16, -16,  -1,  17,  -2,   1,
                  -9, -26,  -9, -10,  -2,  -4,   3,  -3,
                 -14,   2, -11,  -2,  -5,   2,  14,   5,
                 -35,  -8,  11,   2,   8,  15,  -3,   1,
                  -1, -18,  -9,  10, -15, -25, -31, -50,
            ],
            
            [ // EG
                  -9,  22,  22,  27,  27,  19,  10,  20,
                 -17,  20,  32,  41,  58,  25,  30,   0,
                 -20,   6,   9,  49,  47,  35,  19,   9,
                   3,  22,  24,  45,  57,  40,  57,  36,
                 -18,  28,  19,  47,  31,  34,  39,  23,
                 -16, -27,  15,   6,   9,  17,  10,   5,
                 -22, -23, -30, -16, -16, -23, -36, -32,
                 -33, -28, -22, -43,  -5, -32, -20, -41,
            ]
        ],
    
        [ // King
            [ // MG
                 -65,  23,  16, -15, -56, -34,   2,  13,
                  29,  -1, -20,  -7,  -8,  -4, -38, -29,
                  -9,  24,   2, -16, -20,   6,  22, -22,
                 -17, -20, -12, -27, -30, -25, -14, -36,
                 -49,  -1, -27, -39, -46, -44, -33, -51,
                 -14, -14, -22, -46, -44, -30, -15, -27,
                   1,   7,  -8, -64, -43, -16,   9,   8,
                 -15,  36,  12, -54,   8, -28,  24,  14,
            ],
    
            [ // EG
                 -74, -35, -18, -18, -11,  15,   4, -17,
                 -12,  17,  14,  17,  17,  38,  23,  11,
                  10,  17,  23,  15,  20,  45,  44,  13,
                  -8,  22,  24,  27,  26,  33,  26,   3,
                 -18,  -4,  21,  24,  27,  23,   9, -11,
                 -19,  -3,  11,  21,  23,  16,   7,  -9,
                 -27, -11,   4,  13,  14,   4,  -5, -17,
                 -53, -34, -21, -11, -28, -14, -24, -43,
            ]
        ]
    ];

    pub const MATERIAL: [[i32; 2]; 6] = [
        [  82,  94], // Pawn
        [ 477, 512], // Rook
        [ 337, 281], // Knight
        [ 365, 297], // Bishop
        [1025, 936], // Queen
        [   0,   0], // King
    ];

}
