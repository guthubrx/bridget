import { copyFile, mkdir, readFile, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const uiRoot = resolve(scriptDirectory, "..");
const vendorRoot = resolve(uiRoot, "vendor");
const checkOnly = process.argv.includes("--check");

const files = [
  {
    packageName: "echarts",
    version: "6.1.0",
    license: "Apache-2.0",
    source: "node_modules/echarts/dist/echarts.min.js",
    destination: "echarts.min.js",
  },
  {
    packageName: "tabulator-tables",
    version: "6.5.2",
    license: "MIT",
    source: "node_modules/tabulator-tables/dist/js/tabulator.min.js",
    destination: "tabulator.min.js",
  },
  {
    packageName: "tabulator-tables",
    version: "6.5.2",
    license: "MIT",
    source: "node_modules/tabulator-tables/dist/css/tabulator.min.css",
    destination: "tabulator.min.css",
  },
];

const legalFiles = [
  {
    source: "node_modules/echarts/LICENSE",
    destination: "LICENSE.echarts.txt",
  },
  {
    source: "node_modules/echarts/NOTICE",
    destination: "NOTICE.echarts.txt",
  },
  {
    source: "node_modules/tabulator-tables/LICENSE",
    destination: "LICENSE.tabulator.txt",
  },
];

async function digest(filePath) {
  return createHash("sha256").update(await readFile(filePath)).digest("hex");
}

async function main() {
  const manifestFiles = [];
  for (const file of files) {
    const sourcePath = resolve(uiRoot, file.source);
    const destinationPath = resolve(vendorRoot, file.destination);
    const sourceDigest = await digest(sourcePath);
    if (checkOnly) {
      const destinationDigest = await digest(destinationPath);
      if (sourceDigest !== destinationDigest) {
        throw new Error(`${file.destination} ne correspond plus à ${file.source}`);
      }
    } else {
      await mkdir(dirname(destinationPath), { recursive: true });
      await copyFile(sourcePath, destinationPath);
    }
    manifestFiles.push({
      package: file.packageName,
      version: file.version,
      license: file.license,
      source: relative(uiRoot, sourcePath),
      file: `vendor/${file.destination}`,
      sha256: sourceDigest,
    });
  }

  for (const file of legalFiles) {
    const sourcePath = resolve(uiRoot, file.source);
    const destinationPath = resolve(vendorRoot, file.destination);
    if (checkOnly) {
      if ((await digest(sourcePath)) !== (await digest(destinationPath))) {
        throw new Error(`${file.destination} ne correspond plus à ${file.source}`);
      }
    } else {
      await copyFile(sourcePath, destinationPath);
    }
  }

  const manifest = `${JSON.stringify({ version: 1, files: manifestFiles }, null, 2)}\n`;
  const manifestPath = resolve(vendorRoot, "manifest.json");
  if (checkOnly) {
    if ((await readFile(manifestPath, "utf8")) !== manifest) {
      throw new Error("vendor/manifest.json ne correspond plus aux dépendances figées");
    }
  } else {
    await writeFile(manifestPath, manifest, "utf8");
  }
}

await main();
