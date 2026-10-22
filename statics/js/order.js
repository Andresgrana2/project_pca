function orderForm() {
    return {
        form: { weight: '', order_type: '', discount: '' },
        errors: {},
        error: '',
        result: null,
        submitting: false,

        clearFeedback(field) {
            delete this.errors[field];
            this.error = '';
            this.result = null;
        },

        validate() {
            this.errors = {};
            const weight = Number(this.form.weight);

            if (String(this.form.weight).trim() === '') {
                this.errors.weight = 'Ingresa el peso del pedido.';
            } else if (!Number.isFinite(weight) || weight < 0.01 || weight > 20) {
                this.errors.weight = 'El peso debe ser un número entre 0,01 y 20 kg.';
            }
            if (!['STANDARD', 'EXPRESS'].includes(this.form.order_type)) {
                this.errors.order_type = 'Selecciona un tipo de envío válido.';
            }
            if (!['NEW', 'FREQUENT', 'VIP'].includes(this.form.discount)) {
                this.errors.discount = 'Selecciona un tipo de cliente válido.';
            }

            return Object.keys(this.errors).length === 0;
        },

        async submit() {
            if (this.submitting) return;
            this.error = '';
            this.result = null;

            if (!this.validate()) {
                this.$nextTick(() => this.$el.querySelector('.is-invalid')?.focus());
                return;
            }

            this.submitting = true;
            try {
                const response = await fetch('/api/order/new', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json', 'Accept': 'application/json' },
                    body: JSON.stringify({ ...this.form, weight: Number(this.form.weight) }),
                });
                const data = await response.json().catch(() => null);

                if (!response.ok) {
                    throw new Error(typeof data?.error === 'string'
                        ? data.error : 'No se pudo calcular el pedido. Revisa los datos e inténtalo de nuevo.');
                }
                if (!Number.isFinite(data?.total_value)) {
                    throw new Error('El servidor devolvió una respuesta inválida. Inténtalo de nuevo.');
                }

                this.result = data;
            } catch (error) {
                this.error = error instanceof TypeError
                    ? 'No se pudo conectar con el servidor. Revisa tu conexión e inténtalo de nuevo.'
                    : error.message;
            } finally {
                this.submitting = false;
            }
        },

        formatTotal(value) {
            return Number.isFinite(value)
                ? new Intl.NumberFormat('es-CO', { minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(value)
                : '';
        },
    };
}
