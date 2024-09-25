import Spinner from "./components/Spinner";
import { makeOnLoad, SSRProps } from "./lib";
import { useQuery, gql } from '@apollo/client';
import { InstanceDescriptor } from "./model";

const Alter = ({pageData}: SSRProps) => {

    const { loading, error, data } = useQuery<{instance: InstanceDescriptor | null}>(
        gql`
            query Instance($name: String!) {
                instance(name: $name) {
                    name
                    maxMemory
                    port
                }
            }
        `, 
        {
            variables: {
                name: pageData.name
            }
        }
    );

    if (loading) return <Spinner />;
    if (error) return <pre>Error: {error.message}</pre>;

    const instanceData = data!.instance;

    return <p>ALTER {JSON.stringify(instanceData)}</p>
}

window.onload = makeOnLoad(Alter)