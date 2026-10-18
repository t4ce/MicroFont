//! All of These, Raster to just 1 or 2 Quads - and for 66 cells the Cap is that (66)
//! But the actual cap is 33 because of the enemy == ▒  
//! So, suddenly there is 33x more work for the renderer but this is such a small sum of quads
//! overall that any renderer IC will just smile about it anyways

use crate::z128 as z;

pub const BLOCK_SYMBOLS: [(char, u128); 32] = [
    ('▀', z(0x3fffffff, 36)),
    ('▁', 0x3f),
    ('▂', 0x3ffff),
    ('▃', 0xffffff),
    ('▄', 0xfffffffff),
    ('▅', 0x3ffffffffff),
    ('▆', 0xffffffffffff),
    ('▇', 0xfffffffffffffff),
    ('█', 0x3ffffffffffffffff),
    ('▉', 0x3efbefbefbefbefbe),
    ('▊', 0x3efbefbefbefbefbe),
    ('▋', 0x3cf3cf3cf3cf3cf3c),
    ('▌', 0x38e38e38e38e38e38),
    ('▍', 0x30c30c30c30c30c30),
    ('▎', 0x30c30c30c30c30c30),
    ('▏', 0x20820820820820820),
    ('▐', 0x71c71c71c71c71c7),
    ('░', 0x22222222222222222),
    ('▒', 0x2a56a56a56a56a56a),
    ('▓', 0x1dddddddddddddddd),
    ('▔', z(0x3f, 60)),
    ('▕', 0x1041041041041041),
    ('▖', z(0x1c71c71c7, 3)),
    ('▗', 0x1c71c71c7),
    ('▘', z(0x71c71c7, 39)),
    ('▙', 0x38e38e38fffffffff),
    ('▚', 0x38e38e381c71c71c7),
    ('▛', 0x3fffffffe38e38e38),
    ('▜', 0x3fffffff1c71c71c7),
    ('▝', z(0x71c71c7, 36)),
    ('▞', 0x71c71c7e38e38e38),
    ('▟', 0x71c71c7fffffffff),
];
