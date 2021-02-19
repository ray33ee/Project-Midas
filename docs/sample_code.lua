
prime = 67867967

function generate_data(endpoint_index, endpoint_count)

	data = {}

	upper = prime --math.floor(math.sqrt(prime))
	width = math.floor((upper - 2) / endpoint_count)


	data[1] = width * endpoint_index + 2
	data[2] = data[1] + width - 1

	return data

end

function execute_code()

	participant_result = {}

	lower = global_data[1]
	upper = global_data[2]

	for i = lower,upper,1
	do
		if (prime % i == 0 )
		then
			participant_result[1] = i
			return participant_result

		end

	end

	participant_result[1] = 0

	return participant_result

end

function interpret_results()
	for i,v in ipairs(results)
	do
		if (v ~= 0)
		then
			return "The number is not prime (divisible by " .. v .. ")."
		end
	end
	return "The number is prime."
end