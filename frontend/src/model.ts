export type ServerState = "RUNNING" | "STOPPED" | "CRASHED"

export type InstanceDescriptor = {
    name: string,
    mods: string,
    state: ServerState,

    memory: number | null,
    maxMemory: number,
    port: number
}