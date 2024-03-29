PHONY: test

zip:
	rm ../CFD.zip
	zip -qr ../CFD.zip *
	ls -lh ../CFD.zip

upload:
	/usr/local/bin/aws s3 cp ../CFD.zip s3://gmto.starccm/
	/usr/local/bin/aws lambda update-function-code --region us-east-2 --function-name CFD2OPD --s3-bucket gmto.starccm --s3-key CFD.zip

test:
	/usr/local/bin/aws lambda invoke --region us-east-2 --function-name CFD2OPD --invocation-type Event --payload file://inputFile.json outfile

all: zip upload
