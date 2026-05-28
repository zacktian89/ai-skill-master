# Task 1: Bootstrap Tauri + Vue

**Files:**
- Create: `package.json`
- Create: `index.html`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `src/main.ts`
- Create: `src/App.vue`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/lib.rs`
- Create: `src-tauri/capabilities/default.json`

- [x] **Step 1: Scaffold in a temporary sibling directory**

Run from `D:\code`:

```powershell
npm create tauri-app@latest SkillMaster-bootstrap -- --template vue-ts --manager npm
```

Expected: the generator creates `D:\code\SkillMaster-bootstrap` and prints commands that include `npm install` and `npm run tauri dev`.

- [x] **Step 2: Copy scaffold files into the repository**

Run:

```powershell
Copy-Item -LiteralPath 'D:\code\SkillMaster-bootstrap\package.json' -Destination 'D:\code\SkillMaster\package.json'
Copy-Item -LiteralPath 'D:\code\SkillMaster-bootstrap\index.html' -Destination 'D:\code\SkillMaster\index.html'
Copy-Item -LiteralPath 'D:\code\SkillMaster-bootstrap\vite.config.ts' -Destination 'D:\code\SkillMaster\vite.config.ts'
Copy-Item -LiteralPath 'D:\code\SkillMaster-bootstrap\tsconfig.json' -Destination 'D:\code\SkillMaster\tsconfig.json'
Copy-Item -LiteralPath 'D:\code\SkillMaster-bootstrap\src' -Destination 'D:\code\SkillMaster\src' -Recurse
Copy-Item -LiteralPath 'D:\code\SkillMaster-bootstrap\src-tauri' -Destination 'D:\code\SkillMaster\src-tauri' -Recurse
```

Expected: `rg --files` lists Vue and Tauri files alongside `docs/superpowers`.

- [x] **Step 3: Set product identity**

Modify `src-tauri/tauri.conf.json` so the visible app name and identifier are:

```json
{
  "productName": "SkillMaster",
  "identifier": "com.zacktian.skillmaster"
}
```

Keep the rest of the generated file structure intact.

- [x] **Step 4: Install runtime and test dependencies**

Run from `D:\code\SkillMaster`:

```powershell
npm install
npm install lucide-vue-next @tauri-apps/plugin-dialog
npm install -D vitest @vue/test-utils jsdom
Set-Location src-tauri
cargo add serde --features derive
cargo add serde_json
cargo add thiserror
cargo add tauri-plugin-dialog
cargo add tempfile --dev
Set-Location ..
```

Expected: `package-lock.json` exists, `src-tauri/Cargo.lock` exists, and no install command exits with an error.

- [x] **Step 5: Add test scripts**

Modify `package.json` scripts to include:

```json
{
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "test": "vitest run",
    "tauri": "tauri"
  }
}
```

Keep generated scripts that already match these names.

- [x] **Step 6: Verify the clean scaffold**

Run:

```powershell
npm run build
Set-Location src-tauri
cargo test
Set-Location ..
```

Expected: Vue build succeeds and Rust tests pass.

- [x] **Step 7: Commit**

Run:

```powershell
git add package.json package-lock.json index.html vite.config.ts tsconfig.json src src-tauri
git commit -m "chore: scaffold Tauri Vue app"
```

Expected: commit succeeds with scaffold files.

---

Parent plan: [2026-05-28-skillmaster-implementation.md](../2026-05-28-skillmaster-implementation.md)
