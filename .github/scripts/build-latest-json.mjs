import { readFile, readdir, writeFile } from "node:fs/promises";
import { basename, join } from "node:path";

const [assetsJsonPath, signaturesDir, outputPath] = process.argv.slice(2);

if (!assetsJsonPath || !signaturesDir || !outputPath) {
  console.error(
    "Usage: node build-latest-json.mjs <release-assets.json> <signatures-dir> <output>",
  );
  process.exit(2);
}

const packageJson = JSON.parse(await readFile("package.json", "utf8"));
const release = JSON.parse(await readFile(assetsJsonPath, "utf8"));
const assets = release.assets ?? [];
const version = packageJson.version;
const releaseTag = process.env.RELEASE_TAG;
const repository = process.env.GITHUB_REPOSITORY;
const serverUrl = process.env.GITHUB_SERVER_URL ?? "https://github.com";

if (!releaseTag || !repository) {
  console.error("RELEASE_TAG and GITHUB_REPOSITORY must be set.");
  process.exit(2);
}

const expected = {
  macos: `SkillMaster_${version}_universal.app.tar.gz`,
  windows: `SkillMaster_${version}_x64.zip`,
};

const byName = new Map(assets.map((asset) => [asset.name, asset]));
const missing = [
  expected.macos,
  `${expected.macos}.sig`,
  expected.windows,
  `${expected.windows}.sig`,
].filter((name) => !byName.has(name));

if (missing.length > 0) {
  console.error(`Missing updater release assets: ${missing.join(", ")}`);
  process.exit(1);
}

const signatureFiles = await readdir(signaturesDir);

async function readSignature(assetName) {
  const sigFile = signatureFiles.find(
    (file) => basename(file) === `${assetName}.sig`,
  );

  if (!sigFile) {
    throw new Error(`Missing downloaded signature file for ${assetName}`);
  }

  return (await readFile(join(signaturesDir, sigFile), "utf8")).trim();
}

const macSignature = await readSignature(expected.macos);
const windowsSignature = await readSignature(expected.windows);
const publishedAt = release.publishedAt ?? new Date().toISOString();
const notes = release.body || `SkillMaster ${version} release.`;

function downloadUrl(assetName) {
  return `${serverUrl}/${repository}/releases/download/${releaseTag}/${encodeURIComponent(assetName)}`;
}

const manifest = {
  version,
  notes,
  pub_date: publishedAt,
  platforms: {
    "darwin-aarch64": {
      signature: macSignature,
      url: downloadUrl(expected.macos),
    },
    "darwin-x86_64": {
      signature: macSignature,
      url: downloadUrl(expected.macos),
    },
    "windows-x86_64": {
      signature: windowsSignature,
      url: downloadUrl(expected.windows),
    },
  },
};

await writeFile(outputPath, `${JSON.stringify(manifest, null, 2)}\n`);
