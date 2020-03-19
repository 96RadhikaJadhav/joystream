import resolve from "@rollup/plugin-node-resolve";
import commonjs from "@rollup/plugin-commonjs";
import typescript from "rollup-plugin-typescript2";
import babel from "rollup-plugin-babel";
import { DEFAULT_EXTENSIONS } from "@babel/core";
import react from "react";
import reactDom from "react-dom";

import pkg from "./package.json";

export default {
<<<<<<< HEAD
  input: "./src/index.js",
=======
  input: "src/index.ts",
>>>>>>> develop
  output: [
    {
      file: pkg.main,
      format: "cjs",
<<<<<<< HEAD
    },
    {
      file: pkg.module,
      format: "es",
=======
      file: pkg.main,
    },
    {
      format: "esm",
      file: pkg.module,
>>>>>>> develop
    },
  ],
  plugins: [
    resolve({
      extensions: [".js", ".jsx", ".ts", ".tsx"],
      preferBuiltins: true,
    }),
    babel({
      exclude: "../../node_modules",
      extensions: [...DEFAULT_EXTENSIONS, ".ts", ".tsx"],
    }),
    commonjs({
      exclude: "../../node_modules",
      namedExports: {
        react: Object.keys(react),
        "react-dom": Object.keys(reactDom),
      },
    }),
<<<<<<< HEAD
=======
    typescript(),
    babel({
      exclude: ["../../node_modules/**", "node_modules/**"],
      extensions: [...DEFAULT_EXTENSIONS, ".ts", ".tsx"],
    }),
>>>>>>> develop
  ],
};
