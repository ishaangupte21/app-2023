import axios from 'axios';

const http = axios.create({
    baseURL: process.env.EXPO_PUBLIC_AXIOS_BASE_URL
});

export default http;