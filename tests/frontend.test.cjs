// Suite unitaria de la lógica del formulario: utiliza módulos nativos de Node.js.
// Simula la red y los ganchos de Alpine.js; no abre ni renderiza un navegador.
const test = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { pathToFileURL } = require('node:url');

// Carga el código real del formulario para probar el mismo comportamiento que
// utiliza la página, sin copiar su implementación dentro de las pruebas.
const sourcePath = path.resolve(__dirname, '../statics/js/order.js');
const source = fs.readFileSync(sourcePath, 'utf8');
// Crea un formulario independiente por prueba. fetchMock permite controlar las
// respuestas de la API; los valores iniciales describen una cotización válida.
function makeForm(fetchMock = async () => ({ok:true,json:async()=>({total_value:10})})) {
  const context = { fetch:fetchMock, Intl, Number, String, Object, Error, TypeError };
  const factory = vm.runInNewContext(source + '\norderForm;', context,
    { filename:pathToFileURL(sourcePath).href });
  const form = factory();
  form.form = {weight:'2.5',order_type:'STANDARD',discount:'NEW'};
  // Ejecuta inmediatamente el callback y ofrece un campo enfocable simulado.
  // Cada caso puede sustituir estos ganchos para comprobar el enfoque de errores.
  form.$nextTick = callback => callback();
  form.$el = {querySelector:()=>({focus(){}})};
  return form;
}

// BR-02 a BR-04: acepta las fronteras del peso con las seis combinaciones
// válidas de cliente y envío (24 conjuntos de entradas en este test).
test('validates accepted weight boundaries and client/shipping combinations', () => {
  const form = makeForm();
  for (const weight of ['0.01','5','5.01','20']) {
    for (const order_type of ['STANDARD','EXPRESS']) {
      for (const discount of ['NEW','FREQUENT','VIP']) {
        form.form={weight,order_type,discount}; assert.equal(form.validate(),true);
      }
    }
  }
});
// Particiones inválidas de peso: vacío, texto, infinito y pesos fuera de rango.
// Cada rechazo debe dejar un mensaje de texto asociado al campo weight.
test('rejects blank, nonnumeric, nonfinite and out of range weights', () => {
  const form=makeForm();
  for(const weight of ['', ' ', 'abc', 'Infinity', '-1', '0', '0.009', '20.01', '25']) {
    form.form.weight=weight; assert.equal(form.validate(),false);
    assert.equal(typeof form.errors.weight,'string');
  }
});
// Aunque las opciones se manipulen fuera del select, validate debe rechazarlas
// y registrar mensajes independientes para cliente y tipo de envío.
test('rejects unknown client and shipping options', () => {
  const form=makeForm(); form.form.order_type='OTHER'; form.form.discount='OTHER';
  assert.equal(form.validate(),false);
  assert.ok(form.errors.order_type); assert.ok(form.errors.discount);
});
// Al editar un campo se borra su error, el error global y la cotización anterior;
// los mensajes de otros campos deben conservarse para permitir su corrección.
test('clears only edited field feedback and prior result', () => {
  const form=makeForm(); form.errors={weight:'wrong',discount:'missing'};
  form.error='old error'; form.result={total_value:12}; form.clearFeedback('weight');
  assert.equal(form.errors.weight,undefined); assert.equal(form.errors.discount,'missing');
  assert.equal(form.error,''); assert.equal(form.result,null);
});
// Evita peticiones simultáneas: si submitting ya está activo, no llama a fetch.
test('does not submit a request while a submission is active', async () => {
  let calls=0;const form=makeForm(async()=>{calls++;}); form.submitting=true;
  await form.submit(); assert.equal(calls,0); assert.equal(form.submitting,true);
});
// Un formulario inválido no se envía; dirige el enfoque al primer campo marcado
// como .is-invalid para facilitar la corrección del dato.
test('invalid submission focuses the first erroneous field', async () => {
  let calls=0, focused=0;const form=makeForm(async()=>{calls++;});form.form.weight='';
  form.$el={querySelector:selector=>{assert.equal(selector,'.is-invalid');return {focus(){focused++;}};}};
  await form.submit();assert.equal(calls,0);assert.equal(focused,1);assert.equal(form.submitting,false);
});
// La ausencia de un elemento enfocable no debe interrumpir la validación.
test('invalid submission tolerates absence of a focus target',async()=>{
  const form=makeForm();form.form.weight='';form.$el={querySelector:()=>null};
  await form.submit();assert.ok(form.errors.weight);
});
// Arrange/Act/Assert: simula una respuesta válida, ejecuta submit y comprueba
// endpoint, método, JSON con peso numérico y resultado guardado en el formulario.
test('submits numeric weight and displays a successful quotation', async () => {
  let called=0;const form=makeForm(async(url,options)=>{
    called++; assert.equal(url,'/api/order/new'); assert.equal(options.method,'POST');
    assert.equal(options.headers['Content-Type'],'application/json');
    assert.deepEqual(JSON.parse(options.body),{weight:2.5,order_type:'STANDARD',discount:'NEW'});
    return {ok:true,json:async()=>({total_value:10,discount:'NEW',order_type:'STANDARD'})};
  });
  await form.submit();assert.equal(called,1);assert.equal(form.result.total_value,10);
  assert.equal(form.submitting,false);assert.equal(form.error,'');
});
// Si la API rechaza la solicitud, muestra su mensaje de negocio, conserva el
// resultado vacío y libera el bloqueo para que el usuario pueda reintentar.
test('shows API error text and releases submission lock',async()=>{
  const form=makeForm(async()=>({ok:false,json:async()=>({error:'Excede el peso máximo'})}));
  await form.submit();assert.equal(form.error,'Excede el peso máximo');
  assert.equal(form.result,null);assert.equal(form.submitting,false);
});
// Respuestas sin un error de texto utilizable deben producir un mensaje alterno.
test('uses fallback feedback for missing or nontext API error',async()=>{
  for(const data of [null,{error:42},{}]) {
    const form=makeForm(async()=>({ok:false,json:async()=>data}));await form.submit();
    assert.match(form.error,/No se pudo calcular/);assert.equal(form.submitting,false);
  }
});
// Si response.json falla, utiliza el mensaje alterno en lugar de propagar
// el error de sintaxis de la respuesta al estado del formulario.
test('handles a response that is not valid JSON',async()=>{
  const form=makeForm(async()=>({ok:false,json:async()=>{throw new SyntaxError('invalid JSON');}}));
  await form.submit();assert.match(form.error,/No se pudo calcular/);
});
// HTTP exitoso no basta: total_value debe ser un número finito para aceptar
// la cotización; comprueba cuerpo ausente, campo ausente, texto e infinito.
test('rejects successful responses without a finite numeric total',async()=>{
  for(const data of [null,{}, {total_value:'10'},{total_value:Infinity}]) {
    const form=makeForm(async()=>({ok:true,json:async()=>data}));await form.submit();
    assert.match(form.error,/respuesta inválida/);assert.equal(form.result,null);
    assert.equal(form.submitting,false);
  }
});
// Un TypeError de fetch representa aquí un fallo de red y debe dar al usuario
// una indicación de conexión, además de liberar submitting.
test('shows actionable connection feedback for network failure',async()=>{
  const form=makeForm(async()=>{throw new TypeError('fetch failed');});await form.submit();
  assert.match(form.error,/No se pudo conectar/);assert.equal(form.submitting,false);
});
// Los errores que no son TypeError conservan su mensaje para mostrarse en la UI.
test('preserves other error messages',async()=>{
  const form=makeForm(async()=>{throw new Error('Unexpected test failure');});await form.submit();
  assert.equal(form.error,'Unexpected test failure');
});
// Verifica el formato es-CO con dos decimales y evita mostrar valores que no sean
// números finitos; este caso comprueba el texto, no su presentación en el DOM.
test('formats finite amounts and leaves invalid amounts blank',()=>{
  const form=makeForm();assert.equal(form.formatTotal(1234.5),'1.234,50');
  for(const value of [NaN,Infinity,undefined,null,'12'])assert.equal(form.formatTotal(value),'');
});
