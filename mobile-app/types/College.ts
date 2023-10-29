interface College {
  ipedsid: string;
  name: string;
  address: string;
  city: string;
  state: string;
  zip: string;
  geo_point_2d: {
    lon: number;
    lat: number;
  };
  naics_desc: string;
}
