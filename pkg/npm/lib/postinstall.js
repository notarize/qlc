"use strict";

const os = require("os");
const fs = require("fs");
const url = require("url");
const https = require("https");
const path = require("path");
const util = require("util");
const crypto = require("crypto");
const childProc = require("child_process");

const fsChmod = util.promisify(fs.chmod);
const fsUnlink = util.promisify(fs.unlink);
const fsExists = util.promisify(fs.exists);
const fsReadFile = util.promisify(fs.readFile);
const mkdir = util.promisify(fs.mkdir);

const {
  version: VERSION,
  checksumConfig: CHECKSUMS,
} = require("../package.json");
const BIN_PATH = path.join(__dirname, "../bin");

function getTarget() {
  const id = `${os.platform()}-${os.arch()}`;
  switch (id) {
    case "darwin-x64":
      return "x86_64-apple-darwin";
    case "darwin-arm64":
      return "aarch64-apple-darwin";
    case "linux-x64":
      return "x86_64-unknown-linux-musl";
    default:
      throw new Error(`Unsupported platform: ${id}`);
  }
}

function download(fullUrl, dest) {
  return new Promise((resolve, reject) => {
    const outFile = fs.createWriteStream(dest);
    const cleanup = (err) =>
      fsUnlink(dest)
        .catch(console.error)
        .finally(() => {
          reject(err);
        });
    const opts = {
      ...url.parse(fullUrl),
      headers: { Accept: "application/octet-stream" },
    };
    https
      .get(opts, (response) => {
        if (response.statusCode === 302) {
          return download(response.headers.location, dest)
            .then(resolve)
            .catch(reject);
        } else if (response.statusCode !== 200) {
          return cleanup(
            new Error(`Download failed with ${response.statusCode}`)
          );
        }
        response.pipe(outFile);
        outFile.on("finish", () => {
          resolve();
        });
      })
      .on("error", cleanup);
  });
}

function untar(zipPath, destDir) {
  return new Promise((resolve, reject) => {
    const unzipProc = childProc.spawn("tar", ["xf", zipPath, "-C", destDir], {
      stdio: "inherit",
    });
    unzipProc.on("error", reject);
    unzipProc.on("close", (code) => {
      return code === 0
        ? resolve()
        : reject(new Error(`tar exited with ${code}`));
    });
  });
}

async function checksum(tarballPath, target) {
  const expected = CHECKSUMS[target];
  if (!expected) {
    throw new Error(`Missing checksum for ${target}`);
  }
  const actual = crypto
    .createHash("sha256")
    .update(await fsReadFile(tarballPath))
    .digest("hex");
  if (expected !== actual) {
    throw new Error(
      `Checksum integrity check failed: expected ${expected}, got ${actual}`
    );
  }
  return actual;
}

async function main() {
  const target = getTarget();
  const fileName = `qlc-${VERSION}-${target}.tar.gz`;
  const url = `https://github.com/notarize/qlc/releases/download/${VERSION}/${fileName}`;
  await Promise.all([
    fsExists(BIN_PATH).then((exists) => {
      return exists ? Promise.resolve() : mkdir(BIN_PATH);
    }),
    download(url, fileName),
  ]);
  try {
    await checksum(fileName, target);
  } catch (error) {
    await fsUnlink(fileName);
    throw error;
  }
  await untar(fileName, BIN_PATH);
  await Promise.all([
    fsUnlink(fileName),
    fsChmod(path.join(BIN_PATH, "qlc"), "755"),
  ]);
}

main().catch((err) => {
  console.error(`Unhandled error: ${err}`);
  process.exit(1);
});
