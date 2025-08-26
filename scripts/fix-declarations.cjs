// scripts/fix-declarations.cjs
const fg = require("fast-glob");
const fs = require("fs");
const path = require("path");
const { parse } = require("@babel/parser");
const traverse = require("@babel/traverse").default; // note .default in CJS
const generate = require("@babel/generator").default; // note .default in CJS

const ROOT = "src/nextjs/src/ic/declarations";
const files = fg.sync([`${ROOT}/**/index.@(js|ts)`], { absolute: true });

let totalEdits = 0;

for (const file of files) {
  let code = fs.readFileSync(file, "utf8");
  const ast = parse(code, { sourceType: "module", plugins: ["typescript"] });

  let fileEdits = 0;

  traverse(ast, {
    VariableDeclarator(path) {
      const id = path.node.id;
      const init = path.node.init;

      // export const canisterId = "process.env.X"  →  process.env.X
      if (id?.type === "Identifier" && id.name === "canisterId" && init?.type === "StringLiteral") {
        const m = init.value.match(/^process\.env\.([A-Z0-9_]+)$/);
        if (m) {
          path.node.init = {
            type: "MemberExpression",
            object: {
              type: "MemberExpression",
              object: { type: "Identifier", name: "process" },
              property: { type: "Identifier", name: "env" },
              computed: false,
            },
            property: { type: "Identifier", name: m[1] },
            computed: false,
          };
          fileEdits++;
        }
      }

      // export const network = process.env.DFX_NETWORK → …NEXT_PUBLIC_DFX_NETWORK
      if (
        id?.type === "Identifier" &&
        id.name === "network" &&
        init?.type === "MemberExpression" &&
        init.object?.type === "MemberExpression" &&
        init.object.object?.name === "process" &&
        init.object.property?.name === "env" &&
        init.property?.type === "Identifier" &&
        init.property.name === "DFX_NETWORK"
      ) {
        init.property.name = "NEXT_PUBLIC_DFX_NETWORK";
        fileEdits++;
      }
    },
  });

  // Also fix DFX_NETWORK in conditional statements
  traverse(ast, {
    MemberExpression(path) {
      if (
        path.node.object?.type === "MemberExpression" &&
        path.node.object.object?.name === "process" &&
        path.node.object.property?.name === "env" &&
        path.node.property?.type === "Identifier" &&
        path.node.property.name === "DFX_NETWORK"
      ) {
        path.node.property.name = "NEXT_PUBLIC_DFX_NETWORK";
        fileEdits++;
      }
    },
  });

  if (fileEdits > 0) {
    const { code: out } = generate(ast, { retainLines: true, comments: true }, code);
    fs.writeFileSync(file, out);
    totalEdits += fileEdits;
    console.log(`Updated ${path.relative(process.cwd(), file)} (${fileEdits} change${fileEdits > 1 ? "s" : ""})`);
  }
}

console.log(`Done. Total edits: ${totalEdits}`);
