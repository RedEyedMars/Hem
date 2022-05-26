import wasm, { GameState, Socketry, meta, img_ids, xs, ys } from "golems";
import animate, { loadImages } from "./graphics";
import { } from './input';

let prevFrameTime = Date.now();
function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
function computeFrames() {
  const currentFrameTime = new Date();
  const result = currentFrameTime - prevFrameTime;
  if(result > 400) {
    prevFrameTime = currentFrameTime;
  }
  return result / 400;
}
let game_running = true;
window.stop_game = () => {
  game_running = false;
}

window.onload = async () => {
  const hem = window.hem = await wasm();
  let game = await GameState.new();
  let sockets = await Socketry.new();
  const screen = game.screen(0);
  console.log(screen);
  console.log(new Int16Array(hem.memory.buffer, screen[0].x, screen[0].s * 4));
  console.log(new Int16Array(hem.memory.buffer, screen[0].x, screen[0].s * 4)[0]);

  loadImages();
  while(game_running) {
    //console.log(a_meta[0] + "," + a_img_ids[0] +","+ a_xs[0] + "," + a_ys[0]);
    game.set_frames_to_compute(computeFrames());
    if(Socketry.has_socket_events()) {
      sockets = await sockets.handle_socket_events();
    }
    if(!game.run()) {
      break;
    }
    game.render();
    //animate(a_meta, a_img_ids, a_xs, a_ys);
    await sleep();
  }
};
