import typescript from '@rollup/plugin-typescript';
import pkg from './package.json';
const banner = `/**
 * UltraFXR v${pkg.version}
 * Copyright (c) 2019-2020 Dietrich Epp
 * https://www.ultrafxr.us/
 * @license MIT
 */`;
export default {
  input: 'src/index.ts',
  output: {
    dir: 'build',
    format: 'es',
    banner,
  },
  plugins: [
    typescript({
      tsconfig: 'src/tsconfig.json',
    }),
  ],
};
