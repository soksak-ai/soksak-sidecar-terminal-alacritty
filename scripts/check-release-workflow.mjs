#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workflow = fs.readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8");
const manifest = JSON.parse(fs.readFileSync(path.join(root, "sidecar.json"), "utf8"));
const ownerPath = `soksak-sidecars/${manifest.id}`;
const targets = JSON.parse(fs.readFileSync(path.join(root, "release/targets.json"), "utf8"));
const requireText = (value, label) => {
  if (!workflow.includes(value)) throw new Error(`release workflow is missing ${label}: ${value}`);
};
const cargo = fs.readFileSync(path.join(root, "Cargo.toml"), "utf8");
const stage = fs.readFileSync(path.join(root, "stage.sh"), "utf8");
if (!/^edition = "2024"$/m.test(cargo)) throw new Error("Rust packages must use edition 2024");
if (/\bpath\s*=\s*"\.\.\//.test(cargo)) throw new Error("Cargo dependencies must not require sibling checkouts");
if (!cargo.includes('rev = "d8d9b2d3f731cd17c10aeaea71a74b1747ff087e"')) throw new Error("Cargo must pin the terminal sidecar kit commit");
if (!cargo.includes('rev = "71f4ac714a98ad69606a54272f4b73a0b30fe7aa"')) throw new Error("Cargo must pin the terminal contract commit");
requireText("ref: 4c83e41a0aa168bc4c2e11100aba242277c731b6", "platform spec commit");
requireText("package_json_file:", "validator-owned pnpm version");
requireText("node-version-file:", "validator-owned Node version");
if (/path:\s+soksak-(?:kits|contracts)\//.test(workflow)) throw new Error("Cargo dependencies must not be staged as sibling repositories");
if (/node-version:\s*["']?\d/.test(workflow)) throw new Error("release workflow must not hardcode Node");
if (/^\s+version:\s*["']?\d/m.test(workflow) || workflow.includes('with: { version: "')) throw new Error("release workflow must not hardcode pnpm");
requireText(`path: ${ownerPath}`, "owner checkout path");
requireText(`working-directory: ${ownerPath}`, "owner working directory");
requireText(`${ownerPath}/\${{ steps.archive.outputs.asset }}`, "artifact upload path");
requireText(`working-directory: ${ownerPath}/.dependency/soksak-spec`, "validator build directory");
for (const obsolete of ["release/source-dependencies.json", "release/dependencies.json"]) {
  if (fs.existsSync(path.join(root, obsolete))) throw new Error(`${obsolete} is obsolete`);
}
for (const { target, runner } of targets) {
  requireText(`target: ${target}`, "release target");
  requireText(`runner: ${runner}`, "release runner");
}
requireText("release-template/sidecar/build-release.mjs", "canonical release builder");
requireText("release-template/sidecar/validate-with-spec.mjs", "canonical release validator");
requireText("release-template/publish-canonical-release.mjs", "canonical immutable publisher");
requireText("cp dist/sidecar.json package/sidecar.json", "target-specific manifest packaging");
requireText("cp dist/soksak-sidecar-terminal-alacritty* package/dist/", "target-specific executable packaging");
requireText("GH_TOKEN: ${{ steps.release-token.outputs.token }}", "GitHub CLI release token");
if (!stage.includes('staged="$name$ext"')) throw new Error("stage.sh must select the target executable name");
if (/"version":\s*"[0-9]+\.[0-9]+\.[0-9]+"/.test(stage)) throw new Error("stage.sh must not duplicate the sidecar version");
if (!stage.includes('sed "s#\\\"process\\\": \\\"dist/$name\\\"#\\\"process\\\": \\\"dist/$staged\\\"#" sidecar.json')) {
  throw new Error("stage.sh must derive the staged manifest from sidecar.json");
}
for (const duplicate of ["build-release.mjs", "release-contract.mjs", "validate-with-spec.mjs"]) {
  if (fs.existsSync(path.join(root, "scripts", duplicate))) throw new Error(`local spec copy is forbidden: scripts/${duplicate}`);
}
if (fs.existsSync(path.join(root, "validation/spec-validator.json"))) throw new Error("local spec pin copy is forbidden");
for (const file of ["scripts/gate.sh", ".github/workflows/release.yml"]) {
  const source = fs.readFileSync(path.join(root, file), "utf8");
  for (const obsolete of ["SOKSAK_PTYD_BIN", "SOKSAK_CORE_WORKTREE", "soksak-ptyd", "vsterm-tauri"]) {
    if (source.includes(obsolete)) throw new Error(`${file} contains obsolete PTY path ${obsolete}`);
  }
}
if (fs.existsSync(path.join(root, "scripts/e2e/ptyd-integration.sh"))) throw new Error("obsolete PTY source-build harness still exists");
console.log("release workflow contract: passed");
