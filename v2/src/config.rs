use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    /// The port the webserver will listen on. Default: 3030
    #[envconfig(from = "PORT", default = "3030")]
    pub port: u16,

    /// The Time-To-Live for the cache after which it will be refreshed in seconds. Default: 600
    #[envconfig(from = "TTL", default = "600")]
    pub ttl: u32,
}
