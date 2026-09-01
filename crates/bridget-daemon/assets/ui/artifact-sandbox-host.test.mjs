import test from "node:test";
import assert from "node:assert/strict";
import sandbox from "./artifact-sandbox-host.js";

test("sandbox accepte seulement une page locale et lui lie son instance", () => {
  const url = sandbox.sandboxFrameUrl("/v1/artifacts/sandbox/frame?ticket=abc", "frame-1");
  assert.equal(url, "/v1/artifacts/sandbox/frame?ticket=abc&frame_instance_id=frame-1");
  assert.equal(sandbox.sandboxFrameUrl("https://example.com/frame", "frame-1"), "");
});
test("protocole refuse les messages inconnus, URL et hauteur excessive", () => {
  assert.equal(sandbox.validMessage({ type: "sandbox.resize", frame_instance_id: "x", height: 1200 }, "x"), true);
  assert.equal(sandbox.validMessage({ type: "sandbox.resize", frame_instance_id: "x", height: 1201 }, "x"), false);
  assert.equal(sandbox.validMessage({ type: "sandbox.state", frame_instance_id: "x", ui_state: { url: "file:///tmp/no" } }, "x"), false);
  assert.equal(sandbox.validMessage({ type: "tauri.invoke", frame_instance_id: "x" }, "x"), false);
  assert.equal(sandbox.validMessage({ type: "sandbox.open_source", frame_instance_id: "x", source_ref_id: "source-1" }, "x"), true);
  assert.equal(sandbox.validMessage({ type: "sandbox.open_source", frame_instance_id: "x", source_ref_id: "https://example.org" }, "x"), false);
});
