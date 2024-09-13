pub use pallet_dex_v2 as dex;

// declare pallet id for dex
parameter_types! {
    // pallet ID
    pub const DexPallet: PalletId = PalletId(*b"DCitadel");
}

/// Configure the dex pallet
impl pallet_dex::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type NativeBalance = Balances;
    type Fungibles = Assets;
    type PalletId = DexPallet;
}
