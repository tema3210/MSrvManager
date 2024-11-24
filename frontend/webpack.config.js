// webpack.config.js
const path = require('path');

module.exports = {
  mode: "production",
  entry: {
    'validate': {
      import: [
        'ajv',
        'ajv-formats',
      ],
      dependOn: 'shared',
    },
    'shared': {
      import: [
        'react',
        'react-dom',
        '@apollo/client',
        'styled-components',
        'react-hook-form',
        "@hookform/resolvers",
        './src/components/UIComps.tsx',
      ]
    },
    index: {
      dependOn: 'shared',
      import: './src/Index.tsx',
    },
    create: {
      dependOn: ['shared', 'validate'],
      import: './src/Create.tsx',
    },
    alter: {
      dependOn: ['shared', 'validate'],
      import: './src/Alter.tsx',
    },
    error: {
      dependOn: 'shared',
      import: './src/Error.tsx',
    },
    rcon: {
      dependOn: 'shared',
      import: './src/Rcon.tsx',
    },
    renew: {
      dependOn: ['shared', 'validate'],
      import: './src/Renew.tsx',
    },

  },
  output: {
    path: path.resolve(__dirname, '../static'),
    filename: '[name].js',
  },
  watchOptions: {
    aggregateTimeout: 350, // Delay the rebuild after the first change (in ms)
    ignored: /node_modules/, // Ignore files in `node_modules`
    // poll: 1000, // Use polling instead of filesystem events (useful for network filesystems)
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
