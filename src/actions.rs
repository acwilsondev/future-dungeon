#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    MovePlayer(i16, i16),
    PickUpItem,
    OpenInventory,
    OpenHelp,
    OpenLogHistory,
    OpenBestiary,
    TryLevelTransition,

    // UI Actions
    CloseMenu,
    MenuUp,
    MenuDown,
    MenuSelect,
    ToggleShopMode,

    Wait,
    Target,
}
