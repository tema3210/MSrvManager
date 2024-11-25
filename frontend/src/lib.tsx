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
  uri: `ws://${location.host}/graphql_ws`,
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
  connectToDevTools: true,
  devtools: {
    name: "main"
  }
});

export type SSRProps<T = any> = {
  pageData: T
};

// Wrapper for ApolloProvider
export function Wrapper<T>({ Component }: { Component: React.ComponentType<SSRProps<T>> }): JSX.Element {
  let pageData = window.pageData;
  return <ApolloProvider client={client}><Component pageData={pageData}/></ApolloProvider>;
};

interface OnLoad {
  (): void
}

// OnLoad handler to render the app
export function makeOnLoad<T = any>(C: React.ComponentType<SSRProps<T>>): OnLoad {
  return () => {
    const app = document.getElementById('app');
    if (app) {
      const root = createRoot(app);
      root.render(<Wrapper Component={C} />);
    }
  };
} 
