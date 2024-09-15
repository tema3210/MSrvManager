// webpack.config.js
const path = require('path');

module.exports = {
  mode: "production",
  entry: {
    components: {
      import: ['./web_components.tsx','react','react-dom']
    }, // Use the dynamically generated entry file
    index: { import: './src/Index.tsx', dependOn: 'components'}
  },
  output: {
    path: path.resolve(__dirname, '../static'),
    filename: '[name].js',
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
