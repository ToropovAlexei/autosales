

class ApiError extends Error {
  response: Response;

  constructor(message: string, response: Response) {
    super(message);
    this.response = response;
  }
}

const getAuthToken = () => {
  if (typeof window !== 'undefined') {
    return localStorage.getItem('jwt');
  }
  return null;
};

const handleResponse = async (response: Response) => {
  if (response.status === 204) {
    return null;
  }

  const json = await response.json();

  if (!response.ok) {
    if (response.status === 401 || response.status === 403) {
      localStorage.removeItem('jwt');
      if (typeof window !== 'undefined') {
        window.location.href = '/login';
      }
    }
    const errorMessage = json.error || `API request failed with status ${response.status}`;
    throw new ApiError(errorMessage, response);
  }

  if (json.success === false) {
    throw new ApiError(json.error || 'API request failed', response);
  }

  return json.data;
};

const api = {
  async get(endpoint: string) {
    const token = getAuthToken();
    const response = await fetch(`/api${endpoint}`, {
      headers: {
        'Content-Type': 'application/json',
        ...(token && { 'Authorization': `Bearer ${token}` }),
      },
    });
    return handleResponse(response);
  },

  async post(endpoint: string, data: any) {
    const token = getAuthToken();
    const response = await fetch(`/api${endpoint}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        ...(token && { 'Authorization': `Bearer ${token}` }),
      },
      body: JSON.stringify(data),
    });
    return handleResponse(response);
  },

  async postForm(endpoint: string, data: any) {
    const token = getAuthToken();
    const formData = new URLSearchParams();
    for (const key in data) {
      formData.append(key, data[key]);
    }

    const response = await fetch(`/api${endpoint}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
        ...(token && { 'Authorization': `Bearer ${token}` }),
      },
      body: formData,
    });
    return handleResponse(response);
  },

  async put(endpoint: string, data: any) {
    const token = getAuthToken();
    const response = await fetch(`/api${endpoint}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
        ...(token && { 'Authorization': `Bearer ${token}` }),
      },
      body: JSON.stringify(data),
    });
    return handleResponse(response);
  },

  async delete(endpoint: string) {
    const token = getAuthToken();
    const response = await fetch(`/api${endpoint}`, {
      method: 'DELETE',
      headers: {
        'Content-Type': 'application/json',
        ...(token && { 'Authorization': `Bearer ${token}` }),
      },
    });
    return handleResponse(response);
  },

  async getTransactions() {
    return this.get('/transactions');
  },

  async getStockMovements() {
    return this.get('/stock/movements');
  },
};

export default api;

