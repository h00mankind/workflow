Quick Look previews do not execute JavaScript in HTML artifacts.

For JavaScript-driven single-file prototypes in `docs/artifacts/`, Quick Look thumbnails can confirm static shell styling but not dynamic mounts, event handlers, or state updates. Use a real browser render for interaction checks before reporting the artifact as verified.
