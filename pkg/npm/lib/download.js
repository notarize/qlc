"use strict";

const { unlink, createWriteStream } = require("fs");
const { URL } = require("url");

const introspectionQuery = `
  query IntrospectionQuery {
    __schema {
      queryType { name }
      mutationType { name }
      subscriptionType { name }
      types {
        ...FullType
      }
    }
  }
  fragment FullType on __Type {
    kind
    name
    description
    fields(includeDeprecated: true) {
      name
      description
      args {
        ...InputValue
      }
      type {
        ...TypeRef
      }
    }
    inputFields {
      ...InputValue
    }
    interfaces {
      ...TypeRef
    }
    enumValues(includeDeprecated: true) {
      name
      description
    }
    possibleTypes {
      ...TypeRef
    }
  }
  fragment InputValue on __InputValue {
    name
    description
    type { ...TypeRef }
    defaultValue
  }
  fragment TypeRef on __Type {
    kind
    name
    ofType {
      kind
      name
      ofType {
        kind
        name
        ofType {
          kind
          name
          ofType {
            kind
            name
            ofType {
              kind
              name
              ofType {
                kind
                name
                ofType {
                  kind
                  name
                }
              }
            }
          }
        }
      }
    }
  }
`;

function download() {
  const [,, endpoint, outFilePathRaw] = process.argv;
  if (!endpoint) {
    return Promise.reject(new Error("Endpoint argument is required to download a schema"));
  }
  const outFilePath = outFilePathRaw || "schema.json";
  return new Promise((resolve, reject) => {
    const outFile = createWriteStream(outFilePath);
    const cleanup = (err) => {
      return unlink(outFilePath, (deleteErr) => {
        console.error(deleteErr);
        reject(err);
      });
    };
    const parsedUrl = new URL(endpoint);
    const opts = {
      method: "POST",
      headers: { "Content-Type": "application/json" },
    };
    const requestModule = parsedUrl.protocol === "https:" ? require("https") : require("http");
    requestModule
      .request(parsedUrl, opts, (response) => {
        if (response.statusCode !== 200) {
          return cleanup(new Error(`Download failed with ${response.statusCode}`));
        }
        response.pipe(outFile);
        outFile.on("finish", resolve);
      })
      .on("error", cleanup)
      .end(JSON.stringify({ query: introspectionQuery }));
  });
}

download().catch((err) => {
  console.error(err);
  process.exit(1);
});
