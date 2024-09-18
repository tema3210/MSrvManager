import {InstanceDescriptor} from "../model";

const Desc = ({instance}: {instance: InstanceDescriptor}) => {
    return <p>{JSON.stringify(instance)}</p>
}


export default Desc;