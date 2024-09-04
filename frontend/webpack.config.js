// webpack.config.js
const path = require('path');

module.exports = {
  mode: "production",
  entry: './entry.tsx',  // Use the dynamically generated entry file
  output: {
    path: path.resolve(__dirname, '../static'),
    filename: 'bundle.js',
  },
  module: {
    rules: [
      {
        test: /\.(js|jsx|ts|tsx)$/,
        exclude: /node_modules/,
        use: {
          loader: 'babel-loader',
          options: {
            presets: [
                "@babel/preset-env",
                ['@babel/preset-react', { "runtime": "automatic" }],
                "@babel/preset-typescript"
            ],
            plugins: [
                //
            ],
          },
        },
      },
    ],
  },
  resolve: {
    extensions: ['.js', '.jsx','.ts','.tsx'],
  },
};
