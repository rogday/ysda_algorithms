#include <stdio.h>

#include <cuda_runtime.h>
#include <cstdint>

#include <chrono>
#include <iostream>

#include <helper_cuda.h>

constexpr uint8_t TARGET[11] = "NLXGI4NoAp";
constexpr uint8_t ALPHABET[63] = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
constexpr size_t FORWARD_BY = 100'000'000 + 310;
constexpr size_t STATE_SIZE = 31;
constexpr int64_t MODULO = 2'147'483'647ll;

__global__ void
bruteforce(const uint32_t* forward_matrix, const int64_t* pows, size_t begin)
{
    constexpr uint8_t TARGET[11] = "NLXGI4NoAp";
    constexpr uint8_t ALPHABET[63] = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    constexpr size_t FORWARD_BY = 100'000'000 + 310;
    constexpr size_t STATE_SIZE = 31;
    constexpr int64_t MODULO = 2'147'483'647ll;

    size_t threads = gridDim.x * gridDim.y * blockDim.x * blockDim.y;

    size_t blockId = (gridDim.x * blockIdx.y) + blockIdx.x;
    size_t index = (blockId * (blockDim.x * blockDim.y)) + (threadIdx.y * blockDim.x) + threadIdx.x;

    uint32_t max = 0xFFFFFFFF;
    uint32_t part = max / threads;

    for (uint32_t seed = index * part; seed < (index + 1) * part; ++seed) {
        bool found = true;
        for (size_t i = 0; i < 10; ++i) {
            const uint32_t* coeffs = &forward_matrix[((begin + i) - 10) * STATE_SIZE];
            uint32_t sum = coeffs[0] *
                static_cast<uint32_t>(static_cast<int64_t>(static_cast<int32_t>(seed)) * pows[0]);

            for (size_t j = 1; j < STATE_SIZE; ++j) {
                sum += coeffs[j] * 
                    static_cast<uint32_t>((static_cast<int64_t>(seed) * pows[j]) % MODULO);
            }         

            if (TARGET[i] != ALPHABET[static_cast<size_t>(sum >> 1) % 62]) {
                found = false;
                break;
            }
        }
        if (found) {
            printf("found %lu\n", seed);
        }
    }
}

__global__ void
fast_forward(uint32_t* forward_matrix)
{
    size_t blockId = (gridDim.x * blockIdx.y) + blockIdx.x;
    //size_t thread_id = (blockId * (blockDim.x * blockDim.y)) + (threadIdx.y * blockDim.x) + threadIdx.x;
    size_t thread_id = threadIdx.x;

    constexpr size_t FORWARD_BY = 100'000'000 + 310;
    constexpr size_t STATE_SIZE = 31;
    constexpr uint64_t M_THREADS = STATE_SIZE*3;

    uint64_t work = FORWARD_BY * STATE_SIZE / M_THREADS;

    //printf("dims: %lu %lu %lu, %lu\n", blockDim.x, blockDim.y, gridDim.x, gridDim.y);

    for (int64_t id = thread_id; work-- != 0; id += M_THREADS) {
        int i = id / STATE_SIZE;
        int j = id % STATE_SIZE;

        int start = (3 + i) % STATE_SIZE;
        int finish = (0 + i) % STATE_SIZE;
        forward_matrix[start * STATE_SIZE + j] += forward_matrix[finish * STATE_SIZE + j];
        //printf("%lu : %lu \n", id, thread_id);
        __syncthreads();
    }

}


int
main(void)
{
    int64_t pows[STATE_SIZE] = { 1 };
    for (size_t i = 1; i < STATE_SIZE; ++i) {
        pows[i] = (pows[i - 1] * 16'807) % MODULO;
    }

    uint32_t forward_matrix[STATE_SIZE * STATE_SIZE]{};
    for (size_t i = 0; i < STATE_SIZE; ++i) {
        forward_matrix[i * STATE_SIZE + i] = 1;
    }

    size_t begin = 3;
    size_t end = 0;


    constexpr uint64_t M_THREADS = STATE_SIZE*3;
    uint64_t work = FORWARD_BY * STATE_SIZE / M_THREADS;

    // Error code to check return values for CUDA calls
    cudaError_t err = cudaSuccess;


    // Allocate the device input vector A
    uint32_t* d_forward = NULL;
    err = cudaMalloc((void**)&d_forward, STATE_SIZE * STATE_SIZE * sizeof(uint32_t));

    if (err != cudaSuccess)
    {
        fprintf(stderr, "Failed to allocate device vector A (error code %s)!\n", cudaGetErrorString(err));
        exit(EXIT_FAILURE);
    }

    printf("Copy input data from the host memory to the CUDA device\n");
    //err = cudaMemcpy(d_forward, test_matrix, STATE_SIZE * STATE_SIZE * sizeof(uint32_t), cudaMemcpyHostToDevice);
    err = cudaMemcpy(d_forward, forward_matrix, STATE_SIZE * STATE_SIZE * sizeof(uint32_t), cudaMemcpyHostToDevice);

    if (err != cudaSuccess)
    {
        fprintf(stderr, "Failed to copy vector A from host to device (error code %s)!\n", cudaGetErrorString(err));
        exit(EXIT_FAILURE);
    }

    printf("forwarding");
    fflush(stdout);

    // Launch the Vector Add CUDA Kernel

    auto t1 = std::chrono::system_clock::now();

    fast_forward <<<1, M_THREADS >>> (d_forward);
    cudaDeviceSynchronize();

    auto t2 = std::chrono::system_clock::now();
    std::chrono::duration<double> diff = t2 - t1;
    std::cout << diff.count() << " s\n";

    err = cudaGetLastError();

    if (err != cudaSuccess)
    {
        fprintf(stderr, "Failed to launch vectorAdd kernel (error code %s)!\n", cudaGetErrorString(err));
        exit(EXIT_FAILURE);
    }

    printf("Copy output data from the CUDA device to the host memory\n");
    //err = cudaMemcpy(test_matrix, d_forward, STATE_SIZE * STATE_SIZE * sizeof(uint32_t), cudaMemcpyDeviceToHost);
    err = cudaMemcpy(forward_matrix, d_forward, STATE_SIZE * STATE_SIZE * sizeof(uint32_t), cudaMemcpyDeviceToHost);


    if (err != cudaSuccess)
    {
        fprintf(stderr, "Failed to copy vector C from device to host (error code %s)!\n", cudaGetErrorString(err));
        exit(EXIT_FAILURE);
    }

    //cuda leftovers
    for (int64_t id = work* M_THREADS; id < FORWARD_BY * STATE_SIZE; ++id) {
        int i = id / STATE_SIZE;
        int j = id % STATE_SIZE;

        int start = (3 + i) % STATE_SIZE;
        int finish = (0 + i) % STATE_SIZE;
        //test_matrix[start * STATE_SIZE + j] += test_matrix[finish * STATE_SIZE + j];
        forward_matrix[start * STATE_SIZE + j] += forward_matrix[finish * STATE_SIZE + j];
    }

    

    //for (size_t i = 0; i < STATE_SIZE * STATE_SIZE; ++i) {
     //   printf("%lu %lu\n", forward_matrix[i], test_matrix[i]);
      //  if (forward_matrix[i] != test_matrix[i]) {
      //      return -1;
       // }
   // }



    // Allocate the device input vector A
    uint32_t*d_matrix = NULL;
    err = cudaMalloc((void **)&d_matrix, STATE_SIZE*STATE_SIZE*sizeof(uint32_t));

    if (err != cudaSuccess)
    {
        fprintf(stderr, "Failed to allocate device vector A (error code %s)!\n", cudaGetErrorString(err));
        exit(EXIT_FAILURE);
    }

    // Allocate the device input vector B
    int64_t*d_pows = NULL;
    err = cudaMalloc((void **)&d_pows, STATE_SIZE * sizeof(int64_t));

    if (err != cudaSuccess)
    {
        fprintf(stderr, "Failed to allocate device vector B (error code %s)!\n", cudaGetErrorString(err));
        exit(EXIT_FAILURE);
    }

    // Copy the host input vectors A and B in host memory to the device input vectors in
    // device memory
    printf("Copy input data from the host memory to the CUDA device\n");
    err = cudaMemcpy(d_matrix, forward_matrix, STATE_SIZE * STATE_SIZE * sizeof(uint32_t), cudaMemcpyHostToDevice);

    if (err != cudaSuccess)
    {
        fprintf(stderr, "Failed to copy vector A from host to device (error code %s)!\n", cudaGetErrorString(err));
        exit(EXIT_FAILURE);
    }

    err = cudaMemcpy(d_pows, pows, STATE_SIZE * sizeof(int64_t), cudaMemcpyHostToDevice);

    if (err != cudaSuccess)
    {
        fprintf(stderr, "Failed to copy vector B from host to device (error code %s)!\n", cudaGetErrorString(err));
        exit(EXIT_FAILURE);
    }

    begin = 17;
    printf("running %lu\n", begin);
    fflush(stdout);

    // Launch the Vector Add CUDA Kernel

     t1 = std::chrono::system_clock::now();

    bruteforce <<<196, 256>>>(d_matrix, d_pows, begin);

    cudaDeviceSynchronize();

     t2 = std::chrono::system_clock::now();
     diff = t2 - t1;
    std::cout << diff.count() << " s\n";


    err = cudaGetLastError();

    if (err != cudaSuccess)
    {
        fprintf(stderr, "Failed to launch vectorAdd kernel (error code %s)!\n", cudaGetErrorString(err));
        exit(EXIT_FAILURE);
    }

    // Free device global memory
    err = cudaFree(d_forward);

    if (err != cudaSuccess)
    {
        fprintf(stderr, "Failed to free device vector A (error code %s)!\n", cudaGetErrorString(err));
        exit(EXIT_FAILURE);
    }

    // Free device global memory
    err = cudaFree(d_matrix);

    if (err != cudaSuccess)
    {
        fprintf(stderr, "Failed to free device vector A (error code %s)!\n", cudaGetErrorString(err));
        exit(EXIT_FAILURE);
    }

    err = cudaFree(d_pows);

    if (err != cudaSuccess)
    {
        fprintf(stderr, "Failed to free device vector B (error code %s)!\n", cudaGetErrorString(err));
        exit(EXIT_FAILURE);
    }

    printf("Done\n");
    return 0;
}

