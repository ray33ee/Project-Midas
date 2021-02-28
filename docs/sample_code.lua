
prime = 67867967

function generate_data(endpoint_index, endpoint_count)

	data = {}

	upper = prime --math.floor(math.sqrt(prime))
	width = math.floor((upper - 2) / endpoint_count)

	data.lower = width * endpoint_index + 2
	data.upper = data.lower + width - 1

	return data

end

function execute_code()


	participant_result = {}



	participant_result.lower = global_data.lower
	participant_result.upper = global_data.upper

	lower = global_data.lower
	upper = global_data.upper

	for i = lower,upper,1
	do
        if i % 100000 == 0 then _check() end
	    --if i % 100000 == 0 then _progress(i) end

		if (prime % i == 0 )
		then
			participant_result.divisor = i

		end

	end

	participant_result.divisor = 0

	return participant_result

end

function interpret_results()
	for i,v in pairs(results)
	do
	    if (v.divisor ~= 0)
	    then
	        print("The number is divisible by ".. v.divisor)
	    end

	end
	return "The number is prime."
end