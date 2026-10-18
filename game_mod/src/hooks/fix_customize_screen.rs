// By default the game expects unlocked parts to be sequantial, and will
// incorrectly cycle through them if there are any gaps. Also it assumes
// the original is always unlocked. So patch it to fill in the first available
// parts when entering the screen, and fix the cycling logic so it selects
// the next unlocked part
pub fn fix_customize_screen() {}
