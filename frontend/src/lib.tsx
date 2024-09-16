import { createRoot } from 'react-dom/client';
import { ApolloProvider, ApolloClient, InMemoryCache } from "@apollo/client";

const client = new ApolloClient({
    uri: "/graphql",
    cache: new InMemoryCache()
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