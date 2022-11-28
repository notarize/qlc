import { Buffer } from "node:buffer";
import { promises as fsPromises } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { graphql, buildSchema, getIntrospectionQuery } from "graphql";

const THIS_FILES_DIRNAME = dirname(fileURLToPath(import.meta.url));

function readSchemaGraphQLDefintition() {
  const schemaFilePath = join(THIS_FILES_DIRNAME, "./schema.graphql");
  return fsPromises.readFile(schemaFilePath, "utf8");
}

function getIntroSpectionResultFromDefinition(definition) {
  const schema = buildSchema(definition);
  return graphql({ schema, source: getIntrospectionQuery() });
}

async function writeIntrospectionToFixtureJSON(result) {
  const jsonFilePath = join(THIS_FILES_DIRNAME, "./output/schema.json");
  const json = JSON.stringify(result, null, 2);
  const buffer = Buffer.from(json, "utf8");
  await fsPromises.writeFile(jsonFilePath, buffer);
  return { kb: buffer.byteLength / 1_000, location: jsonFilePath };
}

async function main() {
  const schemaDefinition = await readSchemaGraphQLDefintition();
  const result = await getIntroSpectionResultFromDefinition(schemaDefinition);
  if (!result || result.errors?.length) {
    throw result.errors;
  }
  return writeIntrospectionToFixtureJSON(result);
}

const report = await main();
console.log(`New ${report.kb} kilobytes schema written at ${report.location}!`);
