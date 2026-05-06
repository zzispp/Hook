import packageJson from '../package.json';

// ----------------------------------------------------------------------

type ConfigType = {
  basePath?: string;
  appVersion: string;
  cors: {
    allowedOrigins: string[];
    methods: string[];
  };
};

export const CONFIG: ConfigType = {
  appVersion: packageJson.version,
  basePath:
    process.env.NODE_ENV === 'production' ? process.env.PRODUCTION_API : process.env.DEV_API,
  cors: {
    /**
     * [] = allow all origins
     * ['http://localhost:8081', 'http://localhost:8082'] = allow only these origins
     */
    allowedOrigins: [],
    methods: ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS'],
  },
};
