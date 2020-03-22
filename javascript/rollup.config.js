import typescript from '@rollup/plugin-typescript';
import pkg from './package.json';
const banner = `/**
 * UltraFXR v${pkg.version}
 * Copyright (c) 2019-2020 Dietrich Epp
 * https://www.ultrafxr.us/
 * @license MIT
 */`;
const plugins = [
  typescript({
    tsconfig: 'src/tsconfig.json',
  }),
];
const esConfig = {
  input: {
    'ultrafxr.es': 'src/index.ts',
  },
  output: {
    dir: 'dist',
    format: 'es',
    banner,
  },
  plugins,
};
const cjsConfig = {
  input: {
    'ultrafxr': 'src/index.ts',
  },
  output: {
    dir: 'dist',
    format: 'cjs',
    banner,
  },
  plugins,
};
export default [esConfig, cjsConfig];
