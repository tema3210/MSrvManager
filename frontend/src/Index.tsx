import { makeOnLoad } from "./lib";

import { gql, useMutation, useSubscription } from "@apollo/client";

const Index = ({}) => {

    const { data, loading, error } = useSubscription(gql`
        subscription  {
            servers {
                name,
                memory,
                maxMemory
            }
        }
      `);

    const urlToZip = "https://drive.google.com/file/d/1g1ug7Fr9yH7RxfqBx5rFyeOHRXENTeZY/view?usp=drive_link";

    const mutationQ = gql`
        mutation Mutation($driveUrl: Url!) {
            newServer(name: "test", cmds: {
                up: "echo hi",
                setup: "echo bye"
            }, url: $driveUrl, maxMemory: 1.5, port: 25565, rcon: 26001)
        }
    `;

    const [create, mr] = useMutation(
        mutationQ,
        {
            variables: {
                driveUrl: urlToZip
            }
        }
    );

    if (loading) return "Loading server list";
    if (error) return <pre>{error.message}</pre>

    return <>
        <p>Hello, we have {JSON.stringify(data)}</p>
        <button onClick={() => create()}>test create server</button>
        {/* <p>the mutation results: {JSON.stringify(mr)}</p> */}
    </>
}

window.onload = makeOnLoad(<Index />)
