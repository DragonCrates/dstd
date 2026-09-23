// This example shows how to create a native TLS variable shim from C++

typedef void (*Destructor)(void*);

class Tls {
    public:
    void *value = nullptr;
    Destructor destroy = nullptr;
    ~Tls() {
        if(value && destroy) destroy(value);
    }
};

thread_local Tls slot;

extern "C" void Tls_set_destructor(Destructor destroy) {
    slot.destroy = destroy;
}

extern "C" void Tls_set(void *value) {
    slot.value = value;
}

extern "C" void *Tls_get() {
    return slot.value;
}
