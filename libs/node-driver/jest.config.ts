export default {
  preset: 'ts-jest',
  testEnvironment: 'node',
  bail: 1,
  verbose: true,
  maxWorkers: 1,
  testTimeout: 5_000_000,
  testMatch: ['**/?(*.)+(spec|test).[jt]s?(x)'],
  /* bazel copies files using symlinks */
  /* jest doesn't like symlinks by default */
  /* enable symlinks and disable watchman to use symlinks */
  haste: {
    enableSymlinks: true,
  },
  watchman: false,
};
