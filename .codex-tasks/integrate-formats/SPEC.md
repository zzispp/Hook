# Integrate Formats Crate

Goal: copy Aether aether-ai-formats into Hook as crates/formats, make it compile in this workspace, and wire it into the existing proxy crate without removing the current conversion path in the same change.

Non-goals: full runtime replacement of every Hook conversion call site; broad provider transport migration.
