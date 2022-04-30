import { push_event, InputEvent } from "golems";

let previousFrameTime = new Date();

window.addEventListener("keydown", e => {
  push_event(InputEvent.key(e.keyCode));
});
window.addEventListener("keyup", e => {
  push_event(InputEvent.unkey(e.keyCode));
});
export function onMouseDown(x,y) {
  push_event(InputEvent.md(x,y));
}
export function onMouseUp(x,y) {
    push_event(InputEvent.mu(x,y));
}
export function onMouseHover(x,y) {
    push_event(InputEvent.hover(x,y));
}
export function onMouseExit() {
    push_event(InputEvent.unhover());
}
