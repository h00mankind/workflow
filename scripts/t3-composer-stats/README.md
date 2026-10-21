# t3-composer-stats

Adds a stats row below the T3 Code composer. The left side shows the usage windows of the provider (5h, Week). The right side shows the context use of the open thread and the output speed of its last turn.

The color of each usage bar shows the usage that is left. Green means more than half is left. Amber means half or less is left. Red means a fifth or less is left.

## Use

Quit T3 Code. Then run the script from a terminal that is not inside T3 Code.

```sh
node scripts/t3-composer-stats/patch.mjs            # apply, or apply again
node scripts/t3-composer-stats/patch.mjs --status   # report the state
node scripts/t3-composer-stats/patch.mjs --restore  # remove the patch
```

The default target is `/Applications/T3 Code (Nightly).app`. Set `T3_APP` to use a different bundle.

A Nightly update replaces the bundle. Run the script again after each update.

## How it works

`patch.mjs` puts `inject.js` into `index.html` of the web client inside `app.asar`. It writes a new archive header and adds the new `index.html` after the existing data. All other files keep their offsets. The script does not change other files in the bundle.

`inject.js` reads the JSON frames that the client and the server exchange on the WebSocket. It does not use minified names. It depends on the wire protocol and on two DOM anchors. `form[data-chat-composer-form]` finds the composer. `.shadow-composer` finds the composer card, and the row goes below that card. If the card is not found, the row goes at the end of the form.

- Usage windows come from `usageLimits` in the provider status.
- Context comes from `contextUsage` and `tokenUsage` of the thread.
- tok/s is the output tokens of the last completed turn, divided by the generation time. The generation time starts at the first reasoning or message item. It does not include the time that tools ran.

The script keeps the context and tok/s values in `localStorage` under `t3-composer-stats:v1`, because the server does not send old turns again after a restart.

## Limits

- The patch breaks the code signature seal of the bundle. The app starts because its asar integrity fuse is off. If a future build turns that fuse on, the patched app will not start. Use `--restore`, or install the app again.
- macOS can block the write to `/Applications`. Allow your terminal in System Settings > Privacy & Security > App Management.
- tok/s is an approximate value for the last completed turn. It is not a live value.
- Context and tok/s show for a thread only after its next turn.
- A provider that reports no usage windows shows no bars.
- `--restore` removes the patch but does not make the archive smaller. Each apply adds approximately 12 kB.
