"use strict";

const { platform, arch } = require("os");
const { createWriteStream } = require("fs");
const { mkdir, unlink, chmod, readFile, access } = require("fs/promises");
const url = require("url");
const { get: httpsGet } = require("https");
const { join } = require("path");
const { createHash } = require("crypto");
const { spawn } = require("child_process");

const {
  version: VERSION,
  checksumConfig: CHECKSUMS,
} = require("../package.json");
const BIN_PATH = join(__dirname, "../bin");

function getTarget() {
  const id = `${platform()}-${arch()}`;
  switch (id) {
    case "darwin-x64":
      return "x86_64-apple-darwin";
    case "darwin-arm64":
      return "aarch64-apple-darwin";
    case "linux-x64":
      return "x86_64-unknown-linux-musl";
    case "linux-arm64":
      return "aarch64-unknown-linux-musl";
    default:
      throw new Error(`Unsupported platform: ${id}`);
  }
}

function download(fullUrl, dest) {
  return new Promise((resolve, reject) => {
    const cleanup = (err) =>
      unlink(dest)
        .catch(console.error)
        .finally(() => {
          reject(err);
        });
    const opts = {
      ...url.parse(fullUrl),
      timeout: 20000,
      headers: { Accept: "application/octet-stream" },
    };
    httpsGet(opts, (response) => {
      switch (response.statusCode) {
        case 200:
          return response.pipe(createWriteStream(dest).on("finish", resolve));
        case 302:
          return download(response.headers.location, dest)
            .then(resolve)
            .catch(reject);
        default:
          return cleanup(
            new Error(`Download failed with ${response.statusCode}`),
          );
      }
    })
      .on("error", cleanup)
      .end();
  });
}

function untar(zipPath, destDir) {
  return new Promise((resolve, reject) => {
    spawn("tar", ["xf", zipPath, "-C", destDir], { stdio: "inherit" })
      .on("error", reject)
      .on("close", (code) => {
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
  const actual = createHash("sha256")
    .update(await readFile(tarballPath))
    .digest("hex");
  if (expected !== actual) {
    throw new Error(
      `Checksum integrity check failed: expected ${expected}, got ${actual}`,
    );
  }
  return actual;
}

async function main() {
  const target = getTarget();
  const tarballFileName = `qlc-${VERSION}-${target}.tar.gz`;
  const url = `https://github.com/notarize/qlc/releases/download/${VERSION}/${tarballFileName}`;
  await Promise.all([
    access(BIN_PATH).catch(() => mkdir(BIN_PATH)),
    download(url, tarballFileName),
  ]);
  try {
    await checksum(tarballFileName, target);
  } catch (error) {
    await unlink(tarballFileName);
    throw error;
  }
  await untar(tarballFileName, BIN_PATH);
  await Promise.all([
    unlink(tarballFileName),
    chmod(join(BIN_PATH, "qlc"), "755"),
  ]);
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error(`Unhandled error: ${err}`);
    process.exit(1);
  });
