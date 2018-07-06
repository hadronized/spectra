let s = new WebSocket("ws://localhost:1234");

const EMPTY_MODE = JSON.stringify("empty_mode");

function send_empty_mode() {
  s.send(EMPTY_MODE);
}

function send_shadertoy_mode(shader_key) {
  let mode = { 'shader_toy': shader_key };
  s.send(JSON.stringify(mode));
}

on_input_empty_mode = send_empty_mode;

function on_input_shadertoy_mode() {
  send_shadertoy_mode('shaders.test');
}
