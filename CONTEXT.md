# Vinyl

Vinyl is a visual novel engine for writers, game developers, and engine integrators.

## Language

**Run**:
Launch a project as an interactive, rendered desktop game with visual and audio presentation.
_Avoid_: Smoke test, simulation, browser preview

**Vinyl CLI**:
The `vn` desktop command used to create, validate, inspect, smoke-test, and run Vinyl projects.
_Avoid_: vn_cli

**Start Label**:
The required `start` story label where every project begins, independent of file discovery order.
_Avoid_: First file, first statement

**Smoke**:
Execute a project's story deterministically without opening its audiovisual presentation, for automated verification and debugging.
_Avoid_: Run, play

**Default Player UI**:
Vinyl's ready-to-use 16:9 presentation for projects that do not provide custom presentation: background, character sprites, dialogue, choices, transitions, music, and pause-based save/load screens. It preserves composition across desktop window sizes and starts the story automatically when initial content is ready.
_Avoid_: Theme system, editor UI, branded splash screen

**Save Slot**:
One of twelve player-selected records containing story state, presentation, screenshot, and save time. Each project also maintains one separate autosave.
_Avoid_: Quick-save, unlimited history

**Fade**:
A transition through a blank frame: the old scene disappears before the new scene appears.
_Avoid_: Crossfade, dissolve

**Dissolve**:
A crossfade where old and new visuals overlap while their opacity changes in opposite directions.
_Avoid_: Fade
