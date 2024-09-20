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

export type SSRProps = {
  pageData: any
};

// Wrapper for ApolloProvider
export const Wrapper = ({ Component }: { Component: React.ComponentType<SSRProps> }) => {
  let pageData = window.pageData;
  return <ApolloProvider client={client}><Component pageData={pageData}/></ApolloProvider>;
};

// OnLoad handler to render the app
export const makeOnLoad = (C: React.ComponentType<SSRProps>) => () => {
  const app = document.getElementById('app');
  if (app) {
    const root = createRoot(app);
    root.render(<Wrapper Component={C} />);
  }
};
