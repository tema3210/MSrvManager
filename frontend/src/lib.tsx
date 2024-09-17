import { createRoot } from 'react-dom/client';
import { ApolloClient, InMemoryCache, split, HttpLink, ApolloProvider } from '@apollo/client';
import { WebSocketLink } from '@apollo/client/link/ws';
import { getMainDefinition } from '@apollo/client/utilities';

// HTTP link for queries/mutations
const httpLink = new HttpLink({
  uri: '/graphql',
});

// WebSocket link for subscriptions
const wsLink = new WebSocketLink({
  uri: `ws://${location.hostname}/graphql`,
  options: {
    reconnect: true,
  },
});

// Use WebSocket for subscriptions and HTTP for queries/mutations
const splitLink = split(
  ({ query }) => {
    const definition = getMainDefinition(query);
    return (
      definition.kind === 'OperationDefinition' &&
      definition.operation === 'subscription'
    );
  },
  wsLink,
  httpLink
);

const client = new ApolloClient({
    link: splitLink,
    cache: new InMemoryCache(),
});

export const Wrapper = ({component}: {component: React.ReactNode}) => {
    return <ApolloProvider client={client}>
        {component}
    </ApolloProvider>
}

export const makeOnLoad = (c: React.ReactElement) => () => {
    const app = document.getElementById('app');
    if (app) {
        const root = createRoot(app);
        root.render(<Wrapper component={c}/>);
    }
}