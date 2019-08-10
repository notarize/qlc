"use strict";

const os = require("os");
const fs = require("fs");
const https = require("https");
const path = require("path");
const util = require("util");
const childProc = require("child_process");

const fsChmod = util.promisify(fs.chmod);
const fsUnlink = util.promisify(fs.unlink);
const fsExists = util.promisify(fs.exists);
const mkdir = util.promisify(fs.mkdir);

const VERSION = require("../package.json").version;
const BIN_PATH = path.join(__dirname, "../bin");

function getTarget() {
  switch (os.platform()) {
    case "darwin":
      return "x86_64-apple-darwin";
    case "linux":
      return "x86_64-unknown-linux-musl";
    default:
      throw new Error(`Unsupported platform: ${os.platform()}`);
  }
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    const outFile = fs.createWriteStream(dest);
    const cleanup = (err) => fsUnlink(dest)
      .catch(console.error)
      .finally(() => {
        reject(err);
      });
    const headers = { Accept: "application/octet-stream" };
    https.get(url, { headers }, (response) => {
      if (response.statusCode === 302) {
        return download(response.headers.location, dest)
          .then(resolve).catch(reject);
      } else if (response.statusCode !== 200) {
        return cleanup(new Error(`Download failed with ${response.statusCode}`));
      }
      response.pipe(outFile);
      outFile.on("finish", () => {
        resolve();
      });
    }).on("error", cleanup);
  });
}

function untar(zipPath, destDir) {
  return new Promise((resolve, reject) => {
    const unzipProc = childProc.spawn(
      "tar",
      ["xf", zipPath, "-C", destDir],
      { stdio: "inherit" },
    );
    unzipProc.on("error", reject);
    unzipProc.on("close", code => {
      return code === 0 ? resolve() : reject(new Error(`tar exited with ${code}`));
    });
  });
}

async function main() {
  const fileName = `qlc-${VERSION}-${getTarget()}.tar.gz`;
  const url = `https://github.com/notarize/qlc/releases/download/${VERSION}/${fileName}`;
  await Promise.all([
    fsExists(BIN_PATH).then(exists => {
      return exists ? Promise.resolve() : mkdir(BIN_PATH);
    }),
    download(url, fileName),
  ]);
  await untar(fileName, BIN_PATH);
  await Promise.all([
    fsUnlink(fileName),
    fsChmod(path.join(BIN_PATH, "qlc"), "755"),
  ]);
}

main()
  .catch(err => {
    console.error(`Unhandled error: ${err}`);
    process.exit(1);
  });
