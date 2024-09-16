import { makeOnLoad } from "./lib";

import { useQuery, gql } from "@apollo/client";

const Index = ({}) => {

    const { data, loading, error } = useQuery(gql`
        {
          servers {
            name,
            memory,
            maxMemory
          }
        }
      `);

    if (loading) return "Loading...";
    if (error) return <pre>{error.message}</pre>

    return <>Hello, we have {JSON.stringify(data)}</>
}

window.onload = makeOnLoad(<Index />)
