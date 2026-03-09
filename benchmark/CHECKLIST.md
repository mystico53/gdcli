# Scoring Checklist

Score each item 1 (pass) or 0 (fail). Total: /20

## Structure (5 pts)
- [ ] project.godot exists and is valid
- [ ] main_menu.tscn exists
- [ ] game.tscn exists
- [ ] settings.tscn exists
- [ ] credits.tscn exists

## Main Menu Scene (5 pts)
- [ ] Has Control root node
- [ ] Has background ColorRect
- [ ] Has title Label with text containing "Game"
- [ ] Has Play, Settings, Quit buttons
- [ ] Buttons are inside a VBoxContainer (or similar layout container)

## Placeholder Scenes (3 pts)
- [ ] game.tscn has identifying label + back button
- [ ] settings.tscn has identifying label + back button
- [ ] credits.tscn has identifying label + back button

## Navigation (5 pts)
- [ ] Play button transitions to game.tscn
- [ ] Settings button transitions to settings.tscn
- [ ] Quit button calls get_tree().quit() or equivalent
- [ ] game.tscn back button returns to main menu
- [ ] settings.tscn back button returns to main menu

## Runs Without Errors (2 pts)
- [ ] Project opens in Godot editor without errors
- [ ] `godot --headless --quit` exits cleanly (no script errors on load)

## Scoring
- 18-20: Excellent
- 14-17: Good
- 10-13: Partial
- <10: Failed
