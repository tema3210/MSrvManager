import { createRoot } from 'react-dom/client';
import React from 'react';
import {
  ApolloClient,
  InMemoryCache,
  split,
  ApolloProvider
} from '@apollo/client';
import { WebSocketLink } from '@apollo/client/link/ws';
import { getMainDefinition } from '@apollo/client/utilities';
//@ts-ignore
import createUploadLink from 'apollo-upload-client/createUploadLink.mjs';

// WebSocket link for subscriptions
const wsLink = new WebSocketLink({
  uri: `ws://${location.hostname}/graphql_ws`,
  options: {
    reconnect: true,
  },
});

const formDataLink = createUploadLink({
  uri: '/graphql'
});

// Use WebSocket for subscriptions and the custom formDataLink for other operations
const splitLink = split(
  ({ query }) => {
    const definition = getMainDefinition(query);
    return (
      definition.kind === 'OperationDefinition' &&
      definition.operation === 'subscription'
    );
  },
  wsLink,
  formDataLink
);

// Initialize Apollo Client
const client = new ApolloClient({
  link: splitLink,
  cache: new InMemoryCache(),
});

// Wrapper for ApolloProvider
export const Wrapper = ({ component }: { component: React.ReactNode }) => {
  return <ApolloProvider client={client}>{component}</ApolloProvider>;
};

// OnLoad handler to render the app
export const makeOnLoad = (c: React.ReactElement) => () => {
  const app = document.getElementById('app');
  if (app) {
    const root = createRoot(app);
    root.render(<Wrapper component={c} />);
  }
};
