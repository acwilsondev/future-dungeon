#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    MovePlayer(i16, i16),
    PickUpItem,
    OpenInventory,
    OpenSpells,
    OpenHelp,
    OpenLogHistory,
    OpenBestiary,
    TryLevelTransition,
    Confirm,
    Decline,

    // UI Actions
    CloseMenu,
    MenuUp,
    MenuDown,
    MenuSelect,
    ToggleShopMode,

    Wait,
    Target,

    // Debug Console
    ToggleDebugConsole,
    TypeChar(char),
    Backspace,
    SubmitCommand,
}
