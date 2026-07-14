# Save files preserve rollback history

Vinyl save files include the interaction checkpoints required for rollback, so loading a slot restores the player's recent story navigation rather than starting a new rollback boundary. This expands the save schema but preserves expected visual-novel behavior across save/load.
